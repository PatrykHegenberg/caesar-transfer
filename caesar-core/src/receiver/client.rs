use std::{fs, io::stdout, path::Path};

use crate::shared::{
    packets::{
        packet::Value, ChunkPacket, HandshakePacket, HandshakeResponsePacket, ListPacket, Packet,
        ProgressPacket,
    },
    JsonPacket, JsonPacketResponse, JsonPacketSender, PacketSender, Sender, Socket, Status,
};

use aes_gcm::{aead::Aead, Aes128Gcm, Key};
use base64::{engine::general_purpose, Engine as _};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use hmac::{Hmac, Mac};
use p256::{ecdh::EphemeralSecret, pkcs8::der::Writer, PublicKey};
use prost::Message;
use rand::rngs::OsRng;
use sha2::Sha256;
use tokio_tungstenite::tungstenite::{protocol::Message as WebSocketMessage, Error};
use tracing::error;

const DESTINATION: u8 = 0;
const NONCE_SIZE: usize = 12;

struct File {
    name: String,
    size: u64,
    progress: u64,
    handle: fs::File,
}

struct Context {
    hmac: Vec<u8>,
    sender: Sender,
    key: EphemeralSecret,
    shared_key: Option<Aes128Gcm>,
    files: Vec<File>,
    sequence: u32,
    index: usize,
    progress: u64,
    length: u64,
}

fn on_join_room(size: Option<usize>) -> Status {
    if size.is_none() {
        return Status::Err("Invalid join room packet.".into());
    }

    println!("Connected to room.");

    Status::Continue()
}

fn on_error(message: String) -> Status {
    Status::Err(message)
}

fn on_leave_room(context: &mut Context, _: usize) -> Status {
    if context.files.iter().any(|file| file.progress < 100) {
        println!();
        println!("Transfer was interrupted because the host left the room.");

        Status::Err("Transfer was interrupted because the host left the room.".into())
    } else {
        Status::Exit()
    }
}

fn on_list(filepath: String, context: &mut Context, list: ListPacket) -> Status {
    if context.shared_key.is_none() {
        return Status::Err("Invalid list packet: no shared key established".into());
    }

    for entry in list.entries {
        let path = sanitize_filename::sanitize(entry.name.clone());
        let file_path = format!("{}/{}", filepath, path);

        if Path::new(&file_path).exists() {
            return Status::Err(format!("The file '{}' already exists.", path));
        }
        let handle = match fs::File::create(&file_path) {
            Ok(handle) => handle,
            Err(error) => {
                return Status::Err(format!(
                    "Error: Failed to create file '{}': {}",
                    file_path, error
                ));
            }
        };

        let file = File {
            name: entry.name,
            size: entry.size,
            handle,
            progress: 0,
        };

        context.files.push(file);
    }

    context.index = 0;
    context.progress = 0;
    context.sequence = 0;
    context.length = 0;

    Status::Continue()
}

fn on_chunk(context: &mut Context, chunk: ChunkPacket) -> Status {
    if context.shared_key.is_none() {
        return Status::Err("Invalid chunk packet: no shared key established".into());
    }

    if chunk.sequence != context.sequence {
        return Status::Err(format!(
            "Expected sequence {}, but got {}.",
            context.sequence, chunk.sequence
        ));
    }

    let Some(file) = context.files.get_mut(context.index) else {
        return Status::Err("Invalid file index.".into());
    };

    context.length += chunk.chunk.len() as u64;

    context.sequence += 1;

    file.handle.write(&chunk.chunk).unwrap();

    file.progress = (context.length * 100) / file.size;

    if file.progress == 100 || file.progress - context.progress >= 1 || chunk.sequence == 0 {
        context.progress = file.progress;

        let progress = ProgressPacket {
            index: context.index.try_into().unwrap(),
            progress: context.progress.try_into().unwrap(),
        };

        context.sender.send_encrypted_packet(
            &context.shared_key,
            DESTINATION,
            Value::Progress(progress),
        );

        print!("\rTransferring '{}': {}%", file.name, file.progress);
        std::io::Write::flush(&mut stdout()).unwrap();
    }

    if file.size == context.length {
        context.index += 1;
        context.length = 0;
        context.progress = 0;
        context.sequence = 0;

        println!();
    }

    Status::Continue()
}

fn on_handshake(context: &mut Context, handshake: HandshakePacket) -> Status {
    if context.shared_key.is_some() {
        return Status::Err("Already performed handshake.".into());
    }

    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    mac.update(&handshake.public_key);

    let verification = mac.verify_slice(&handshake.signature);
    if verification.is_err() {
        return Status::Err("Invalid signature from the sender.".into());
    }

    let public_key = context.key.public_key().to_sec1_bytes().into_vec();

    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    mac.update(&public_key);

    let signature = mac.finalize().into_bytes().to_vec();
    let shared_public_key = PublicKey::from_sec1_bytes(&handshake.public_key).unwrap();

    let shared_secret = context.key.diffie_hellman(&shared_public_key);
    let shared_secret = shared_secret.raw_secret_bytes();
    let shared_secret = &shared_secret[0..16];

    let shared_key: &Key<Aes128Gcm> = shared_secret.into();
    let shared_key = <Aes128Gcm as aes_gcm::KeyInit>::new(shared_key);

    let handshake_response = HandshakeResponsePacket {
        public_key,
        signature,
    };

    context
        .sender
        .send_packet(DESTINATION, Value::HandshakeResponse(handshake_response));

    context.shared_key = Some(shared_key);

    Status::Continue()
}

fn on_message(filepath: String, context: &mut Context, message: WebSocketMessage) -> Status {
    match message.clone() {
        WebSocketMessage::Text(text) => {
            let packet = match serde_json::from_str(&text) {
                Ok(packet) => packet,
                Err(_) => {
                    return Status::Continue();
                }
            };
            return match packet {
                JsonPacketResponse::Join { size } => on_join_room(size),
                JsonPacketResponse::Leave { index } => on_leave_room(context, index),
                JsonPacketResponse::Error { message } => on_error(message),
                _ => Status::Err(format!("Unexpected json packet: {:?}", packet)),
            };
        }
        WebSocketMessage::Binary(data) => {
            let data = &data[1..];

            let data = if let Some(shared_key) = &context.shared_key {
                let nonce = &data[..NONCE_SIZE];
                let ciphertext = &data[NONCE_SIZE..];

                shared_key.decrypt(nonce.into(), ciphertext).unwrap()
            } else {
                data.to_vec()
            };

            let packet = Packet::decode(data.as_ref()).unwrap();
            let value = packet.value.unwrap();
            return match value {
                Value::List(list) => on_list(filepath, context, list),
                Value::Chunk(chunk) => on_chunk(context, chunk),
                Value::Handshake(handshake) => on_handshake(context, handshake),
                _ => Status::Err(format!("Unexpected packet: {:?}", value)),
            };
        }
        _ => (),
    }

    Status::Err("Invalid message type".into())
}

pub async fn start(filepath: String, socket: Socket, fragment: &str) {
    let Some(index) = fragment.rfind('-') else {
        println!("Error: The invite code '{}' is not valid.", fragment);
        return;
    };

    let id = &fragment[..index];
    let hmac = &fragment[index + 1..];
    let Ok(hmac) = general_purpose::STANDARD.decode(hmac) else {
        error!("Error: Invalid base64 inside the invite code.");
        return;
    };

    let key = EphemeralSecret::random(&mut OsRng);

    let (sender, receiver) = flume::bounded(1000);

    let (outgoing, incoming) = socket.split();

    let mut context = Context {
        hmac,
        sender,
        key,

        shared_key: None,
        files: vec![],

        index: 0,
        sequence: 0,
        progress: 0,
        length: 0,
    };

    println!("Attempting to join room '{}'...", id);

    context
        .sender
        .send_json_packet(JsonPacket::Join { id: id.to_string() });

    let outgoing_handler = receiver.stream().map(Ok).forward(outgoing);
    let incoming_handler = incoming.try_for_each(|message| {
        match on_message(filepath.clone(), &mut context, message) {
            Status::Exit() => {
                context.sender.send_json_packet(JsonPacket::Leave);
                println!("Transfer has completed.");

                return future::err(Error::ConnectionClosed);
            }
            Status::Err(error) => {
                println!("Error: {}", error);

                return future::err(Error::ConnectionClosed);
            }
            _ => {}
        };

        future::ok(())
    });

    pin_mut!(incoming_handler, outgoing_handler);

    future::select(incoming_handler, outgoing_handler).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::tungstenite::Message as WebSocketMessage;

    #[test]
    fn test_on_join_room_valid_size() {
        assert_eq!(on_join_room(Some(10)), Status::Continue());
    }
    #[test]
    fn test_on_join_room_invalid_size() {
        assert_eq!(
            on_join_room(None),
            Status::Err("Invalid join room packet.".into())
        );
    }
    #[test]
    fn test_on_error_with_message() {
        assert_eq!(
            on_error("Error message".to_string()),
            Status::Err("Error message".to_string())
        );
    }
    #[test]
    fn test_on_leave_room() {
        let (sender, _) = flume::bounded(1000);
        let mut context = Context {
            hmac: vec![],
            sender,
            key: EphemeralSecret::random(&mut OsRng),
            shared_key: None,
            files: vec![
                File {
                    name: "file1.txt".to_string(),
                    size: 100,
                    progress: 100,
                    handle: fs::File::create("file1.txt").unwrap(),
                },
                File {
                    name: "file2.txt".to_string(),
                    size: 100,
                    progress: 50,
                    handle: fs::File::create("file2.txt").unwrap(),
                },
            ],
            sequence: 0,
            index: 0,
            progress: 0,
            length: 0,
        };

        assert_eq!(
            on_leave_room(&mut context, 0),
            Status::Err("Transfer was interrupted because the host left the room.".into())
        );
        context.files[1].progress = 100;
        assert_eq!(on_leave_room(&mut context, 0), Status::Exit());
    }
    #[test]
    fn test_on_message_text_join() {
        let (sender, _) = flume::bounded(1000);
        let mut context = Context {
            hmac: vec![],
            sender,
            key: EphemeralSecret::random(&mut OsRng),
            shared_key: None,
            files: vec![],
            sequence: 0,
            index: 0,
            progress: 0,
            length: 0,
        };

        let text_message = WebSocketMessage::Text(r#"{"type":"join","size":10}"#.to_string());
        assert_eq!(
            on_message("".to_string(), &mut context, text_message),
            Status::Continue()
        );
    }

    #[test]
    fn test_on_chunk() {
        let (sender, _) = flume::bounded(1000);
        let mut context = Context {
            hmac: vec![],
            sender,
            key: EphemeralSecret::random(&mut OsRng),
            shared_key: None,
            files: vec![File {
                name: "file1.txt".to_string(),
                size: 100,
                progress: 0,
                handle: fs::File::create("file1.txt").unwrap(),
            }],
            sequence: 0,
            index: 0,
            progress: 0,
            length: 0,
        };
        let chunk_packet = ChunkPacket {
            sequence: 0,
            chunk: b"Hello, world!".to_vec(),
        };
        assert_eq!(
            on_chunk(&mut context, chunk_packet),
            Status::Err("Invalid chunk packet: no shared key established".into())
        );
    }
}
