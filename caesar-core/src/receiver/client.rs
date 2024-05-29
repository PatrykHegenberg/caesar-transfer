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

#[cfg(target_os = "android")]
const FILE_PATH_PREFIX: &str = "/storage/emulated/0/Download";


/// Represents a file to be transferred.
///
/// # Fields
///
/// - `name`: The name of the file.
/// - `size`: The total size of the file in bytes.
/// - `progress`: The number of bytes that have been transferred so far.
/// - `handle`: The file handle for reading and writing the file.
#[derive(Debug)]
struct File {
    /// The name of the file.
    name: String,

    /// The total size of the file in bytes.
    size: u64,

    /// The number of bytes that have been transferred so far.
    progress: u64,

    /// The file handle for reading and writing the file.
    handle: fs::File,
}


/// Represents the state of the receiver.
///
/// # Fields
///
/// - `hmac`: The HMAC key used for authentication.
/// - `sender`: The sender used for sending packets.
/// - `key`: The ephemeral secret key used for key agreement.
/// - `shared_key`: The shared key used for encryption.
/// - `files`: The list of files being transferred.
/// - `sequence`: The sequence number of the last received packet.
/// - `index`: The index of the current file being transferred.
/// - `progress`: The number of bytes transferred so far.
/// - `length`: The total length of the file being transferred.
struct Context {
    /// The HMAC key used for authentication.
    hmac: Vec<u8>,

    /// The sender used for sending packets.
    sender: Sender,

    /// The ephemeral secret key used for key agreement.
    key: EphemeralSecret,

    /// The shared key used for encryption.
    shared_key: Option<Aes128Gcm>,

    /// The list of files being transferred.
    files: Vec<File>,

    /// The sequence number of the last received packet.
    sequence: u32,

    /// The index of the current file being transferred.
    index: usize,

    /// The number of bytes transferred so far.
    progress: u64,

    /// The total length of the file being transferred.
    length: u64,
}


/// Handle the join room packet.
///
/// # Arguments
///
/// * `size` - The size of the room.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error if the join room packet is invalid.
fn on_join_room(size: Option<usize>) -> Status {
    // Check if the size of the room is provided
    if size.is_none() {
        // Return an error if the join room packet is invalid
        return Status::Err("Invalid join room packet.".into());
    }

    // Print a message indicating that the client has successfully connected to the room
    println!("Connected to room.");

    // Return a continue status to indicate that the operation was successful
    Status::Continue()
}


/// Handle the error packet.
///
/// # Arguments
///
/// * `message` - The error message.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error with the provided error message.
fn on_error(message: String) -> Status {
    // Return an error with the provided error message
    Status::Err(message)
}


/// Handle the leave room packet.
///
/// # Arguments
///
/// * `context` - The receiver context.
/// * `_` - The index of the sender. Currently unused.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error if there are still files being transferred.
fn on_leave_room(context: &mut Context, _: usize) -> Status {
    // Check if there are any files being transferred with less than 100% progress
    if context.files.iter().any(|file| file.progress < 100) {
        // Print a message indicating that the transfer was interrupted because the host left the room
        println!();
        println!("Transfer was interrupted because the host left the room.");

        // Return an error with the provided message
        Status::Err("Transfer was interrupted because the host left the room.".into())
    } else {
        // Return an exit status to indicate that the operation was successful
        Status::Exit()
    }
}


/// Handle the list packet.
///
/// # Arguments
///
/// * `filepath` - The path to the directory where the files will be saved.
/// * `context` - The receiver context.
/// * `list` - The list packet containing the files to be transferred.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error if the list packet is invalid or if a file with the same name already exists.
fn on_list(filepath: String, context: &mut Context, list: ListPacket) -> Status {
    // Check if the shared key is established
    if context.shared_key.is_none() {
        return Status::Err("Invalid list packet: no shared key established".into());
    }

    // Iterate over the entries in the list packet
    for entry in list.entries {
        // Sanitize the filename to prevent directory traversal attacks
        let path = sanitize_filename::sanitize(entry.name.clone());
        // Construct the file path
        let file_path = format!("{}/{}", filepath, path);

        // Check if the file already exists
        if Path::new(&file_path).exists() {
            return Status::Err(format!("The file '{}' already exists.", path));
        }

        // Create a new file
        let handle = match fs::File::create(&file_path) {
            Ok(handle) => handle,
            Err(error) => {
                return Status::Err(format!(
                    "Error: Failed to create file '{}': {}",
                    file_path, error
                ));
            }
        };

        // Create a new file object and add it to the context
        let file = File {
            name: entry.name,
            size: entry.size,
            handle,
            progress: 0,
        };

        context.files.push(file);
    }

    // Reset the context for the next file transfer
    context.index = 0;
    context.progress = 0;
    context.sequence = 0;
    context.length = 0;

    Status::Continue()
}

/// Handle a chunk packet.
///
/// This function is responsible for processing chunk packets received from the sender.
/// It checks if the shared key has been established, verifies the sequence number,
/// writes the chunk to the corresponding file, updates the file's progress, sends progress
/// updates if necessary, and handles the end of a file transfer.
///
/// # Arguments
///
/// * `context` - The receiver context.
/// * `chunk` - The chunk packet received from the sender.
///
/// # Returns
///
/// A status indicating if the operation was successful.
fn on_chunk(context: &mut Context, chunk: ChunkPacket) -> Status {
    // Check if the shared key is established
    if context.shared_key.is_none() {
        return Status::Err("Invalid chunk packet: no shared key established".into());
    }

    // Verify the sequence number
    if chunk.sequence != context.sequence {
        return Status::Err(format!(
            "Expected sequence {}, but got {}.",
            context.sequence, chunk.sequence
        ));
    }

    // Get the file corresponding to the current index
    let Some(file) = context.files.get_mut(context.index) else {
        return Status::Err("Invalid file index.".into());
    };

    // Update the file's length
    context.length += chunk.chunk.len() as u64;

    // Increment the sequence number
    context.sequence += 1;

    // Write the chunk to the file
    file.handle.write(&chunk.chunk).unwrap();

    // Update the file's progress
    file.progress = (context.length * 100) / file.size;

    // Send progress updates if necessary
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

    // Handle the end of a file transfer
    if file.size == context.length {
        context.index += 1;
        context.length = 0;
        context.progress = 0;
        context.sequence = 0;

        println!();
    }

    Status::Continue()
}

/// Handle the handshake packet.
///
/// This function is responsible for handling the handshake packet received from the sender.
/// It performs the necessary verification and establishes the shared key between the sender and receiver.
///
/// # Arguments
///
/// * `context` - The receiver context.
/// * `handshake` - The handshake packet received from the sender.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
fn on_handshake(context: &mut Context, handshake: HandshakePacket) -> Status {
    // Check if the shared key is already established
    if context.shared_key.is_some() {
        return Status::Err("Already performed handshake.".into());
    }

    // Create a HMAC instance using the shared key
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    // Update the HMAC with the sender's public key
    mac.update(&handshake.public_key);

    // Verify the signature using the HMAC
    let verification = mac.verify_slice(&handshake.signature);
    if verification.is_err() {
        return Status::Err("Invalid signature from the sender.".into());
    }

    // Generate the receiver's public key
    let public_key = context.key.public_key().to_sec1_bytes().into_vec();

    // Create a new HMAC instance using the shared key
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    // Update the HMAC with the receiver's public key
    mac.update(&public_key);

    // Generate the signature using the HMAC
    let signature = mac.finalize().into_bytes().to_vec();

    // Convert the sender's public key into a `PublicKey` object
    let shared_public_key = PublicKey::from_sec1_bytes(&handshake.public_key).unwrap();

    // Perform Diffie-Hellman key exchange
    let shared_secret = context.key.diffie_hellman(&shared_public_key);
    let shared_secret = shared_secret.raw_secret_bytes();
    let shared_secret = &shared_secret[0..16];

    // Create a new 128-bit AES-GCM key from the shared secret
    let shared_key: &Key<Aes128Gcm> = shared_secret.into();
    let shared_key = <Aes128Gcm as aes_gcm::KeyInit>::new(shared_key);

    // Create the handshake response packet
    let handshake_response = HandshakeResponsePacket {
        public_key,
        signature,
    };

    // Send the handshake response packet to the sender
    context
        .sender
        .send_packet(DESTINATION, Value::HandshakeResponse(handshake_response));

    // Establish the shared key
    context.shared_key = Some(shared_key);

    Status::Continue()
}

/// Handle a message received from the WebSocket connection.
///
/// This function takes a `filepath` string, a mutable reference to a `Context` struct,
/// and a `WebSocketMessage` enum. It returns a `Status` enum.
///
/// # Arguments
///
/// * `filepath` - A string representing the file path.
/// * `context` - A mutable reference to a `Context` struct.
/// * `message` - A `WebSocketMessage` enum.
///
/// # Returns
///
/// A `Status` enum.
fn on_message(filepath: String, context: &mut Context, message: WebSocketMessage) -> Status {
    // Handle text messages
    match message.clone() {
        WebSocketMessage::Text(text) => {
            // Parse the JSON packet
            let packet = match serde_json::from_str(&text) {
                Ok(packet) => packet,
                Err(_) => {
                    return Status::Continue();
                }
            };
            // Handle different types of JSON packets
            return match packet {
                JsonPacketResponse::Join { size } => on_join_room(size),
                JsonPacketResponse::Leave { index } => on_leave_room(context, index),
                JsonPacketResponse::Error { message } => on_error(message),
                _ => Status::Err(format!("Unexpected json packet: {:?}", packet)),
            };
        }
        // Handle binary messages
        WebSocketMessage::Binary(data) => {
            // Extract the data from the binary message
            let data = &data[1..];

            let data = if let Some(shared_key) = &context.shared_key {
                let nonce = &data[..NONCE_SIZE];
                let ciphertext = &data[NONCE_SIZE..];

                shared_key.decrypt(nonce.into(), ciphertext).unwrap()
            } else {
                data.to_vec()
            };

            // Decode the packet
            let packet = Packet::decode(data.as_ref()).unwrap();
            let value = packet.value.unwrap();
            // Handle different types of packets
            return match value {
                Value::List(list) => on_list(filepath, context, list),
                Value::Chunk(chunk) => on_chunk(context, chunk),
                Value::Handshake(handshake) => on_handshake(context, handshake),
                _ => Status::Err(format!("Unexpected packet: {:?}", value)),
            };
        }
        _ => (),
    }

    // Return an error status for invalid message types
    Status::Err("Invalid message type".into())
}

 
/// Starts the receiver's client.
///
/// This function takes in a file path, a socket, and a fragment string. It
/// then extracts the room ID and HMAC from the fragment string. The function
/// also generates an ephemeral secret key.
///
/// The function initializes a `Context` struct with the extracted information
/// and sets up the necessary communication channels. It then sends a join
/// request to the server and starts handling incoming messages.
///
/// # Arguments
///
/// * `filepath` - The path to the file to be received.
/// * `socket` - The WebSocket connection to the server.
/// * `fragment` - The invite code containing the room ID and HMAC.
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
