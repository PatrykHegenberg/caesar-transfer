use crate::sender::http_client::send_info;
use crate::sender::util::{hash_random_name, replace_protocol};
use crate::shared::{
    packets::{
        list_packet, packet::Value, ChunkPacket, HandshakePacket, HandshakeResponsePacket,
        ListPacket, Packet, ProgressPacket,
    },
    JsonPacket, JsonPacketResponse, JsonPacketSender, PacketSender, Sender, Socket, Status,
};

use aes_gcm::{aead::Aead, Aes128Gcm, Key};
use base64::{engine::general_purpose, Engine as _};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use hmac::{Hmac, Mac};
use p256::{ecdh::EphemeralSecret, PublicKey};
use prost::Message;
use rand::{rngs::OsRng, RngCore};
use sha2::Sha256;
use std::{
    fs,
    io::{stdout, Write},
    path::Path,
    time::Duration,
};
// use tokio::sync::mpsc;
use tokio::sync::mpsc;
use tokio::{io::AsyncReadExt, task::JoinHandle, time::sleep};
use tokio_tungstenite::tungstenite::{protocol::Message as WebSocketMessage, Error};
use tracing::{debug, error};

const DESTINATION: u8 = 1;
const NONCE_SIZE: usize = 12;
const MAX_CHUNK_SIZE: isize = u16::MAX as isize;
const DELAY: Duration = Duration::from_millis(750);

#[derive(Clone)]
struct File {
    path: String,
    name: String,
    size: u64,
}

struct Context {
    hmac: Vec<u8>,
    sender: Sender,
    key: EphemeralSecret,
    files: Vec<File>,
    shared_key: Option<Aes128Gcm>,
    task: Option<JoinHandle<()>>,
}

fn on_create_room(
    context: &Context,
    id: String,
    relay: String,
    transfer_name: String,
    is_local: bool,
) -> Status {
    debug!("Creating room on: {relay}");
    let base64 = general_purpose::STANDARD.encode(&context.hmac);
    let url = format!("{}-{}", id, base64);

    let hash_name = hash_random_name(transfer_name.clone());

    let send_url = url.to_string();
    let h_name = hash_name.to_string();
    let server_url = replace_protocol(relay.as_str());
    let res = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(send_info(&server_url, &h_name, send_url.as_str(), is_local))
    })
    .join()
    .unwrap();
    debug!("Got Result: {:?}", res);
    match res {
        Ok(transfer_response) => {
            if !transfer_response.local_room_id.is_empty()
                && !transfer_response.relay_room_id.is_empty()
            {
                println!();

                if let Err(error) = qr2term::print_qr(&transfer_name) {
                    error!("Failed to generate QR code: {}", error);
                }
                println!();

                println!("Created room: {}", url);
                println!("Transfername is: {}", transfer_name);
            }
        }
        Err(e) => {
            error!("Error sending info: {e}");
        }
    }

    Status::Continue()
}

fn on_join_room(context: &Context, size: Option<usize>) -> Status {
    if size.is_some() {
        return Status::Err("Invalid join room packet.".into());
    }

    let public_key = context.key.public_key().to_sec1_bytes().into_vec();

    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();
    mac.update(&public_key);

    let signature = mac.finalize().into_bytes().to_vec();

    let handshake = HandshakePacket {
        public_key,
        signature,
    };

    context
        .sender
        .send_packet(DESTINATION, Value::Handshake(handshake));

    Status::Continue()
}

fn on_error(message: String) -> Status {
    Status::Err(message)
}

fn on_leave_room(context: &mut Context, _: usize) -> Status {
    if let Some(task) = &context.task {
        task.abort();
    }

    context.key = EphemeralSecret::random(&mut OsRng);

    context.shared_key = None;

    context.task = None;

    println!();
    error!("Transfer was interrupted because the receiver disconnected.");

    Status::Continue()
}

fn on_progress(context: &Context, progress: ProgressPacket) -> Status {
    if context.shared_key.is_none() {
        return Status::Err("Invalid progress packet: no shared key established".into());
    }

    let file = match context.files.get(progress.index as usize) {
        Some(file) => file,
        None => return Status::Err("Invalid index in progress packet.".into()),
    };

    print!("\rTransferring '{}': {}%", file.name, progress.progress);
    stdout().flush().unwrap();

    if progress.progress == 100 {
        println!();

        if progress.index as usize == context.files.len() - 1 {
            return Status::Exit();
        }
    }

    Status::Continue()
}

async fn on_chunk(sender: Sender, shared_key: Option<Aes128Gcm>, files: Vec<File>) {
    for file in files {
        let mut sequence = 0;
        let mut chunk_size = MAX_CHUNK_SIZE;
        let mut size = file.size as isize;

        let mut handle = match tokio::fs::File::open(file.path).await {
            Ok(handle) => handle,
            Err(error) => {
                println!("Error: Unable to open file '{}': {}", file.name, error);
                return;
            }
        };

        while size > 0 {
            if size < chunk_size {
                chunk_size = size;
            }

            let mut chunk = vec![0u8; chunk_size.try_into().unwrap()];
            handle.read_exact(&mut chunk).await.unwrap();

            sender.send_encrypted_packet(
                &shared_key,
                DESTINATION,
                Value::Chunk(ChunkPacket { sequence, chunk }),
            );

            sequence += 1;
            size -= chunk_size;
        }

        sleep(DELAY).await;
    }
}

fn on_handshake_finalize(context: &mut Context) -> Status {
    let mut entries = vec![];

    for (index, file) in context.files.iter().enumerate() {
        let entry = list_packet::Entry {
            index: index.try_into().unwrap(),
            name: file.name.clone(),
            size: file.size,
        };

        entries.push(entry);
    }

    context.sender.send_encrypted_packet(
        &context.shared_key,
        DESTINATION,
        Value::List(ListPacket { entries }),
    );

    context.task = Some(tokio::spawn(on_chunk(
        context.sender.clone(),
        context.shared_key.clone(),
        context.files.clone(),
    )));

    Status::Continue()
}

fn on_handshake(context: &mut Context, handshake_response: HandshakeResponsePacket) -> Status {
    if context.shared_key.is_some() {
        return Status::Err("Already performed handshake.".into());
    }

    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    mac.update(&handshake_response.public_key);

    let verification = mac.verify_slice(&handshake_response.signature);
    if verification.is_err() {
        return Status::Err("Invalid signature from the receiver.".into());
    }

    let shared_public_key = PublicKey::from_sec1_bytes(&handshake_response.public_key).unwrap();

    let shared_secret = context.key.diffie_hellman(&shared_public_key);
    let shared_secret = shared_secret.raw_secret_bytes();
    let shared_secret = &shared_secret[0..16];

    let shared_key: &Key<Aes128Gcm> = shared_secret.into();
    let shared_key = <Aes128Gcm as aes_gcm::KeyInit>::new(shared_key);

    context.shared_key = Some(shared_key);

    on_handshake_finalize(context)
}

fn on_message(
    context: &mut Context,
    message: WebSocketMessage,
    relay: String,
    transfer_name: String,
    is_local: bool,
) -> Status {
    match message.clone() {
        WebSocketMessage::Text(text) => {
            let packet = match serde_json::from_str(&text) {
                Ok(packet) => packet,
                Err(_) => {
                    return Status::Continue();
                }
            };
            return match packet {
                JsonPacketResponse::Create { id } => {
                    on_create_room(context, id, relay, transfer_name, is_local)
                }
                JsonPacketResponse::Join { size } => on_join_room(context, size),
                JsonPacketResponse::Leave { index } => on_leave_room(context, index),
                JsonPacketResponse::Error { message } => on_error(message),
            };
        }
        WebSocketMessage::Binary(data) => {
            let data = data[1..].to_vec();

            let data = if let Some(shared_key) = &context.shared_key {
                let nonce = &data[..NONCE_SIZE];
                let ciphertext = &data[NONCE_SIZE..];

                shared_key.decrypt(nonce.into(), ciphertext).unwrap()
            } else {
                data
            };

            let packet = Packet::decode(data.as_ref()).unwrap();
            let value = packet.value.unwrap();

            return match value {
                Value::HandshakeResponse(handshake_response) => {
                    on_handshake(context, handshake_response)
                }
                Value::Progress(progress) => on_progress(context, progress),
                _ => Status::Err(format!("Unexpected packet: {:?}", value)),
            };
        }
        _ => (),
    }

    Status::Err("Invalid message type".into())
}

pub async fn start(
    socket: Socket,
    paths: Vec<String>,
    room_id: Option<String>,
    relay: String,
    transfer_name: String,
    is_local: bool,
) {
    let mut files = vec![];

    for path in paths {
        let handle = match fs::File::open(&path) {
            Ok(handle) => handle,
            Err(error) => {
                error!("Error: Failed to open file '{}': {}", path, error);
                return;
            }
        };

        let metadata = handle.metadata().unwrap();

        if metadata.is_dir() {
            error!("Error: The path '{}' does not point to a file.", path);
            return;
        }

        let name = Path::new(&path).file_name().unwrap().to_str().unwrap();

        let size = metadata.len();

        if size == 0 {
            error!("Error: The file '{}' is empty and cannot be sent.", name);
            return;
        }

        files.push(File {
            name: name.to_string(),
            path,
            size,
        });
    }

    let mut hmac = [0u8; 32];
    OsRng.fill_bytes(&mut hmac);

    let key = EphemeralSecret::random(&mut OsRng);

    let (sender, receiver) = flume::bounded(1000);

    let (outgoing, incoming) = socket.split();

    let mut context = Context {
        sender,
        key,
        files,

        hmac: hmac.to_vec(),
        shared_key: None,
        task: None,
    };

    debug!("Attempting to create room...");

    debug!("With Room-ID: {:?}", room_id);
    context.sender.send_json_packet(JsonPacket::Create {
        id: room_id.clone(),
    });

    let outgoing_handler = receiver.stream().map(Ok).forward(outgoing);

    let incoming_handler = incoming.try_for_each(|message| {
        match on_message(
            &mut context,
            message,
            relay.clone(),
            transfer_name.clone(),
            is_local,
        ) {
            Status::Exit() => {
                // TODO: Signal Exit to the server
                context.sender.send_json_packet(JsonPacket::Leave);
                println!("Transfer has completed.");

                return future::err(Error::ConnectionClosed);
            }
            Status::Err(error) => {
                error!("Error: {}", error);

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
    use aes_gcm::KeyInit;

    // #[test]
    // fn test_on_chunk() {
    //     let (sender, _) = flume::bounded(1000);
    //     let context = Context {
    //         hmac: vec![],
    //         sender,
    //         key: EphemeralSecret::random(&mut OsRng),
    //         shared_key: None,
    //         files: vec![
    //             File {
    //                 name: "file1.txt".to_string(),
    //                 size: 100,
    //                 path: "file1.txt".to_string(),
    //             },
    //             File {
    //                 name: "file2.txt".to_string(),
    //                 size: 100,
    //                 path: "file2.txt".to_string(),
    //             },
    //         ],
    //         task: None,
    //     };
    // }
    #[test]
    fn test_on_progress() {
        let (sender, _) = flume::bounded(1000);
        let context = Context {
            hmac: vec![],
            sender,
            key: EphemeralSecret::random(&mut OsRng),
            shared_key: Some(Aes128Gcm::new(Key::<Aes128Gcm>::from_slice(&[0u8; 16]))),
            files: vec![
                File {
                    name: "file1.txt".to_string(),
                    size: 100,
                    path: "file1.txt".to_string(),
                },
                File {
                    name: "file2.txt".to_string(),
                    size: 100,
                    path: "file2.txt".to_string(),
                },
            ],
            task: None,
        };
        assert_eq!(
            on_progress(
                &context,
                ProgressPacket {
                    index: 0,
                    progress: 50
                }
            ),
            Status::Continue()
        );
    }
    #[test]
    fn test_on_create_room() {
        let (sender, _) = flume::bounded(1000);
        let context = Context {
            hmac: vec![],
            sender,
            key: EphemeralSecret::random(&mut OsRng),
            shared_key: None,
            files: vec![
                File {
                    name: "file1.txt".to_string(),
                    size: 100,
                    path: "file1.txt".to_string(),
                },
                File {
                    name: "file2.txt".to_string(),
                    size: 100,
                    path: "file2.txt".to_string(),
                },
            ],
            task: None,
        };
        assert_eq!(
            on_create_room(
                &context,
                "b531e87d-e51a-4507-94f4-335cbe2d32f3-Nc5skZReq7qJN7INwckyAZLWEEbxsrFfH/692tUNgkM="
                    .to_string(),
                String::from("0.0.0.0:8000"),
                String::from("Test"),
                true,
            ),
            Status::Continue()
        );
    }
    // #[test]
    // fn test_on_join_room(){
    //     let (sender, _) = flume::bounded(1000);
    //     let mut context = Context {
    //         hmac: vec![],
    //         sender: sender,
    //         key: EphemeralSecret::random(&mut OsRng),
    //         shared_key: None,
    //         files: vec![
    //             File {
    //                 name: "file1.txt".to_string(),
    //                 size: 100,
    //                 path: "file1.txt".to_string(),
    //             },
    //             File {
    //                 name: "file2.txt".to_string(),
    //                 size: 100,
    //                 path: "file2.txt".to_string(),
    //             },
    //         ],
    //         task: None,
    //     };
    //     assert_eq!(on_join_room(&context, None), Status::Continue());
    // }
    #[test]
    fn test_on_error() {
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
                    path: "file1.txt".to_string(),
                },
                File {
                    name: "file2.txt".to_string(),
                    size: 100,
                    path: "file2.txt".to_string(),
                },
            ],
            task: None,
        };
        assert_eq!(on_leave_room(&mut context, 5), Status::Continue());
    }
    #[test]
    fn test_on_message() {
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
                    path: "file1.txt".to_string(),
                },
                File {
                    name: "file2.txt".to_string(),
                    size: 100,
                    path: "file2.txt".to_string(),
                },
            ],
            task: None,
        };
        assert_eq!(
            on_message(
                &mut context,
                WebSocketMessage::Text(r#"{"type":"leave","index":5}"#.to_string()),
                String::from("0.0.0.0:8000"),
                String::from("Test"),
                true,
            ),
            Status::Continue()
        );
        assert_eq!(on_message(&mut context, WebSocketMessage::Text(r#"{"type":"create","id":"b531e87d-e51a-4507-94f4-335cbe2d32f3-Nc5skZReq7qJN7INwckyAZLWEEbxsrFfH/692tUNgkM="}"#.to_string()), String::from("0.0.0.0:8000"), String::from("Test"), true), Status::Continue());
        assert_eq!(
            on_message(
                &mut context,
                WebSocketMessage::Text(
                    r#"{"type":"error","message":"Error Message: Test"}"#.to_string()
                ),
                String::from("0.0.0.0:8000"),
                String::from("Test"),
                true
            ),
            Status::Err("Error Message: Test".to_string())
        );
    }
}
