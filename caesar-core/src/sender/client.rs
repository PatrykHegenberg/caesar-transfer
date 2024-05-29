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
use tokio::{io::AsyncReadExt, task::JoinHandle, time::sleep};
use tokio_tungstenite::tungstenite::{protocol::Message as WebSocketMessage, Error};
use tracing::{debug, error};

const DESTINATION: u8 = 1;
const NONCE_SIZE: usize = 12;
const MAX_CHUNK_SIZE: isize = u16::MAX as isize;
const DELAY: Duration = Duration::from_millis(750);


/// Struct representing a file to be sent.
///
/// This struct holds the path, name and size of a file.
#[derive(Clone)]
struct File {
    /// The path of the file to be sent.
    path: String,
    /// The name of the file to be sent.
    name: String,
    /// The size of the file to be sent.
    size: u64,
}

/// The context of a sender.
///
/// This struct holds the necessary information for a sender to send files.
/// It includes the HMAC, the sender, the ephemeral secret, the list of files to
/// be sent, the shared key, and the task handling the sending of the files.
struct Context {
    /// The HMAC used for authentication.
    hmac: Vec<u8>,
    /// The sender used to send packets.
    sender: Sender,
    /// The ephemeral secret used for key exchange.
    key: EphemeralSecret,
    /// The list of files to be sent.
    files: Vec<File>,
    /// The shared key used for encryption.
    shared_key: Option<Aes128Gcm>,
    /// The task handling the sending of the files.
    task: Option<JoinHandle<()>>,
}

/// Handles the create room packet.
///
/// This function is called when a create room packet is received.
/// It creates a room on the specified relay and sends the necessary
/// information to the server.
///
/// # Arguments
///
/// * `context` - The context of the sender.
/// * `id` - The ID of the room.
/// * `relay` - The URL of the relay.
/// * `transfer_name` - The name of the transfer.
/// * `is_local` - A boolean indicating whether the room is local.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
fn on_create_room(
    context: &Context,
    id: String,
    relay: String,
    transfer_name: String,
    is_local: bool,
) -> Status {
    // Debug log the relay URL
    debug!("Creating room on: {relay}");

    // Encode the HMAC key using base64
    let base64 = general_purpose::STANDARD.encode(&context.hmac);

    // Generate the URL for the room
    let url = format!("{}-{}", id, base64);

    // Hash the transfer name
    let hash_name = hash_random_name(transfer_name.clone());

    // Create copies of the necessary variables for the thread
    let send_url = url.to_string();
    let h_name = hash_name.to_string();
    let server_url = replace_protocol(relay.as_str());

    // Spawn a new thread to send the information to the server
    let res = std::thread::spawn(move || {
        // Create a new runtime and block on the current thread
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(send_info(&server_url, &h_name, send_url.as_str(), is_local))
    })
    .join()
    .unwrap();

    // Debug log the result
    debug!("Got Result: {:?}", res);

    // Handle the result of sending the information to the server
    match res {
        Ok(transfer_response) => {
            // Print the room URL and transfer name
            if !transfer_response.local_room_id.is_empty()
                && !transfer_response.relay_room_id.is_empty()
            {
                println!();

                // Print the QR code for the transfer name
                if let Err(error) = qr2term::print_qr(&transfer_name) {
                    error!("Failed to generate QR code: {}", error);
                }
                println!();

                println!("Created room: {}", url);
                println!("Transfername is: {}", transfer_name);
            }
        }
        Err(e) => {
            // Log the error
            error!("Error sending info: {e}");
        }
    }

    // Continue with the operation
    Status::Continue()
}


/// Handle the join room packet.
///
/// This function is responsible for handling the join room packet received from the receiver.
/// It checks if the size of the room is provided and returns an error if it is not. It then
/// generates the public key and signs it with the HMAC key. It sends the handshake packet to
/// the receiver.
///
/// # Arguments
///
/// * `context` - The sender context.
/// * `size` - The size of the room.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error if the join room packet is invalid.
fn on_join_room(context: &Context, size: Option<usize>) -> Status {
    // Check if the size of the room is provided
    if size.is_some() {
        return Status::Err("Invalid join room packet.".into());
    }

    // Generate the public key
    let public_key = context.key.public_key().to_sec1_bytes().into_vec();

    // Generate the signature by signing the public key with the HMAC key
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();
    mac.update(&public_key);
    let signature = mac.finalize().into_bytes().to_vec();

    // Create the handshake packet with the public key and signature
    let handshake = HandshakePacket {
        public_key,
        signature,
    };

    // Send the handshake packet to the receiver
    context
        .sender
        .send_packet(DESTINATION, Value::Handshake(handshake));

    Status::Continue()
}



/// Handles errors by returning a `Status` with the error message.
///
/// # Arguments
///
/// * `message` - The error message.
///
/// # Returns
///
/// A `Status` indicating the error with the error message.
#[allow(clippy::missing_panics_doc)]
#[inline]
fn on_error(message: String) -> Status {
    // Return a `Status` with the error message
    Status::Err(message)
}


/// Handle the leave room packet.
///
/// This function handles the leave room packet by aborting any ongoing task,
/// generating a new random key, clearing the shared key, clearing the task,
/// and printing an error message indicating that the transfer was interrupted
/// because the receiver disconnected.
///
/// # Arguments
///
/// * `context` - The sender context.
/// * `_` - The index of the sender. Currently unused.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error if there are still files being transferred.
#[allow(clippy::missing_panics_doc)]
#[inline]
fn on_leave_room(context: &mut Context, _: usize) -> Status {
    // Abort any ongoing task
    if let Some(task) = &context.task {
        task.abort();
    }

    // Generate a new random key
    context.key = EphemeralSecret::random(&mut OsRng);

    // Clear the shared key
    context.shared_key = None;

    // Clear the task
    context.task = None;

    // Print an error message
    println!();
    error!("Transfer was interrupted because the receiver disconnected.");

    // Return a `Status` to indicate that the operation was successful
    Status::Continue()
}


/// Handle the progress packet.
///
/// # Arguments
///
/// * `context` - The sender context.
/// * `progress` - The progress packet.
///
/// # Returns
///
/// A `Status` representing the result of the operation.
///
/// # Errors
///
/// Returns an error if the progress packet is invalid.
#[allow(clippy::missing_panics_doc)]
#[inline]
fn on_progress(context: &Context, progress: ProgressPacket) -> Status {
    // Check if the shared key is established
    if context.shared_key.is_none() {
        // Return an error if the progress packet is invalid
        return Status::Err("Invalid progress packet: no shared key established".into());
    }

    // Get the file corresponding to the progress packet index
    let file = match context.files.get(progress.index as usize) {
        Some(file) => file,
        None => return Status::Err("Invalid index in progress packet.".into()),
    };

    // Print the progress of the file transfer
    print!("\rTransferring '{}': {}%", file.name, progress.progress);
    // Flush the stdout
    stdout().flush().unwrap();

    // Check if the progress is 100%
    if progress.progress == 100 {
        // Print a newline
        println!();

        // Check if this is the last file being transferred
        if progress.index as usize == context.files.len() - 1 {
            // Return an exit status to indicate that the operation was successful
            return Status::Exit();
        }
    }

    // Return a continue status to indicate that the operation was successful
    Status::Continue()
}


/// Asynchronously transfers the chunks of files to the receiver.
///
/// # Arguments
///
/// * `sender` - The sender object used to send packets.
/// * `shared_key` - The shared key used for encryption.
/// * `files` - The list of files to be transferred.
#[allow(clippy::missing_panics_doc)]
#[inline]
async fn on_chunk(
    sender: Sender,
    shared_key: Option<Aes128Gcm>,
    files: Vec<File>,
) {
    // For each file in the list of files
    for file in files {
        let mut sequence = 0;
        let mut chunk_size = MAX_CHUNK_SIZE;
        let mut size = file.size as isize;

        // Open the file
        let mut handle = match tokio::fs::File::open(file.path).await {
            Ok(handle) => handle,
            Err(error) => {
                // Print an error message if the file cannot be opened
                println!("Error: Unable to open file '{}': {}", file.name, error);
                return;
            }
        };

        // While there are still chunks to be transferred
        while size > 0 {
            // If the remaining size is less than the maximum chunk size
            if size < chunk_size {
                // Set the chunk size to the remaining size
                chunk_size = size;
            }

            // Create a vector to hold the chunk
            let mut chunk = vec![0u8; chunk_size.try_into().unwrap()];

            // Read the chunk from the file
            handle.read_exact(&mut chunk).await.unwrap();

            // Send the encrypted chunk packet to the receiver
            sender.send_encrypted_packet(
                &shared_key,
                DESTINATION,
                Value::Chunk(ChunkPacket { sequence, chunk }),
            );

            // Increment the sequence and decrement the size
            sequence += 1;
            size -= chunk_size;
        }

        // Wait for a delay before starting the next file transfer
        sleep(DELAY).await;
    }
}

/// Finalizes the handshake by sending the list of files to the receiver and
/// starting the file transfer task.
///
/// # Arguments
///
/// * `context` - The mutable context holding the sender, files, and shared key.
///
/// # Returns
///
/// A `Status` indicating the success or failure of the handshake finalization.
fn on_handshake_finalize(context: &mut Context) -> Status {
    // Create a vector of `Entry`s from the files in the context
    let mut entries = vec![];
    for (index, file) in context.files.iter().enumerate() {
        let entry = list_packet::Entry {
            // The index of the file in the context
            index: index.try_into().unwrap(),
            // The name of the file
            name: file.name.clone(),
            // The size of the file
            size: file.size,
        };
        entries.push(entry);
    }

    // Send the encrypted list packet to the receiver
    context.sender.send_encrypted_packet(
        &context.shared_key,
        DESTINATION,
        Value::List(ListPacket { entries }),
    );

    // Spawn the file transfer task and store it in the context
    context.task = Some(tokio::spawn(on_chunk(
        context.sender.clone(),
        context.shared_key.clone(),
        context.files.clone(),
    )));

    Status::Continue()
}

/// Handles the handshake response packet received from the receiver.
///
/// # Arguments
///
/// * `context` - The mutable context holding the sender, files, and shared key.
/// * `handshake_response` - The handshake response packet received from the receiver.
///
/// # Returns
///
/// A `Status` indicating the success or failure of the handshake.
fn on_handshake(context: &mut Context, handshake_response: HandshakeResponsePacket) -> Status {
    // Check if the handshake has already been performed
    if context.shared_key.is_some() {
        return Status::Err("Already performed handshake.".into());
    }

    // Create a HMAC instance with the HMAC key
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    // Update the HMAC with the public key from the handshake response
    mac.update(&handshake_response.public_key);

    // Verify the signature in the handshake response
    let verification = mac.verify_slice(&handshake_response.signature);
    if verification.is_err() {
        // Return an error if the signature is invalid
        return Status::Err("Invalid signature from the receiver.".into());
    }

    // Convert the public key from bytes to a public key instance
    let shared_public_key = PublicKey::from_sec1_bytes(&handshake_response.public_key).unwrap();

    // Derive the shared secret using Diffie-Hellman key exchange
    let shared_secret = context.key.diffie_hellman(&shared_public_key);

    // Extract the raw secret bytes from the shared secret
    let shared_secret = shared_secret.raw_secret_bytes();

    // Take the first 16 bytes of the secret as the shared key
    let shared_secret = &shared_secret[0..16];

    // Convert the shared secret bytes to a `Key<Aes128Gcm>` instance
    let shared_key: &Key<Aes128Gcm> = shared_secret.into();

    // Create a new instance of `Aes128Gcm` with the shared key
    let shared_key = <Aes128Gcm as aes_gcm::KeyInit>::new(shared_key);

    // Set the shared key in the context
    context.shared_key = Some(shared_key);

    // Finalize the handshake by sending the list of files and starting the file transfer task
    on_handshake_finalize(context)
}

/// Handles the incoming message from the WebSocket.
///
/// # Arguments
///
/// * `context` - The mutable context holding the sender and shared key.
/// * `message` - The incoming WebSocket message.
/// * `relay` - The URL of the relay.
/// * `transfer_name` - The name of the transfer.
/// * `is_local` - Whether the transfer is local or not.
///
/// # Returns
///
/// A `Status` indicating the success or failure of handling the message.
fn on_message(
    context: &mut Context,
    message: WebSocketMessage,
    relay: String,
    transfer_name: String,
    is_local: bool,
) -> Status {
    match message.clone() {
        // Handle the text WebSocket message
        WebSocketMessage::Text(text) => {
            // Parse the JSON packet from the text message
            let packet = match serde_json::from_str(&text) {
                Ok(packet) => packet,
                Err(_) => {
                    return Status::Continue();
                }
            };

            // Call the corresponding handler based on the packet type
            return match packet {
                // Handle the `Create` packet
                JsonPacketResponse::Create { id } => {
                    on_create_room(context, id, relay, transfer_name, is_local)
                }
                // Handle the `Join` packet
                JsonPacketResponse::Join { size } => on_join_room(context, size),
                // Handle the `Leave` packet
                JsonPacketResponse::Leave { index } => on_leave_room(context, index),
                // Handle the `Error` packet
                JsonPacketResponse::Error { message } => on_error(message),
            };
        }
        // Handle the binary WebSocket message
        WebSocketMessage::Binary(data) => {
            // Extract the encrypted data from the binary message
            let data = data[1..].to_vec();

            // Decrypt the data using the shared key if available
            let data = if let Some(shared_key) = &context.shared_key {
                let nonce = &data[..NONCE_SIZE];
                let ciphertext = &data[NONCE_SIZE..];

                shared_key.decrypt(nonce.into(), ciphertext).unwrap()
            } else {
                data
            };

            // Decode the packet from the decrypted data
            let packet = Packet::decode(data.as_ref()).unwrap();
            let value = packet.value.unwrap();

            // Call the corresponding handler based on the packet value
            return match value {
                // Handle the `HandshakeResponse` packet
                Value::HandshakeResponse(handshake_response) => {
                    on_handshake(context, handshake_response)
                }
                // Handle the `Progress` packet
                Value::Progress(progress) => on_progress(context, progress),
                // Handle unexpected packets
                _ => Status::Err(format!("Unexpected packet: {:?}", value)),
            };
        }
        // Handle other message types
        _ => (),
    }

    // Return an error for unsupported message types
    Status::Err("Invalid message type".into())
}

/// Starts the sender process.
///
/// # Arguments
///
/// * `socket` - The WebSocket connection.
/// * `paths` - The paths to the files to be sent.
/// * `room_id` - The ID of the room to join.
/// * `relay` - The URL of the relay server.
/// * `transfer_name` - The name of the transfer.
/// * `is_local` - Whether the transfer is local or not.
#[allow(clippy::needless_doctest_main)]
pub async fn start(
    socket: Socket,
    paths: Vec<String>,
    room_id: Option<String>,
    relay: String,
    transfer_name: String,
    is_local: bool,
) {
    // Prepare the files to be sent
    let mut files = vec![];

    for path in paths {
        // Open the file
        let handle = match fs::File::open(&path) {
            Ok(handle) => handle,
            Err(error) => {
                error!("Error: Failed to open file '{}': {}", path, error);
                return;
            }
        };

        let metadata = handle.metadata().unwrap();

        // Check if the path points to a file
        if metadata.is_dir() {
            error!("Error: The path '{}' does not point to a file.", path);
            return;
        }

        let name = Path::new(&path).file_name().unwrap().to_str().unwrap();

        let size = metadata.len();

        // Check if the file is empty
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

    // Generate the HMAC key
    let mut hmac = [0u8; 32];
    OsRng.fill_bytes(&mut hmac);

    // Generate the encryption key
    let key = EphemeralSecret::random(&mut OsRng);

    // Create the flume channels
    let (sender, receiver) = flume::bounded(1000);

    // Split the WebSocket connection
    let (outgoing, incoming) = socket.split();

    // Create the context
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
    // Send the create room packet
    context.sender.send_json_packet(JsonPacket::Create {
        id: room_id.clone(),
    });

    // Handle the incoming WebSocket messages
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
                // Send the leave room packet
                context.sender.send_json_packet(JsonPacket::Leave);
                println!("Transfer has completed.");

                // Return an error
                return future::err(Error::ConnectionClosed);
            }
            Status::Err(error) => {
                error!("Error: {}", error);

                // Return an error
                return future::err(Error::ConnectionClosed);
            }
            _ => {}
        };

        future::ok(())
    });

    pin_mut!(incoming_handler, outgoing_handler);

    // Wait for the incoming or outgoing handlers to complete
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
