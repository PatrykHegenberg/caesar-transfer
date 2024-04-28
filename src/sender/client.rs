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

/// A file that is to be sent.
///
/// This structure contains all the information about a file that is to be
/// sent. It is used to keep track of the files that a user wants to send.
#[derive(Clone)]
struct File {
    /// The path to the file on the file system.
    ///
    /// This is the path to the file on the user's file system. The path is
    /// used to open the file and read its contents.
    path: String,

    /// The name of the file.
    ///
    /// This is the name that the file will have when it is received by the
    /// receiver. This name is used when creating the file on the receiver's
    /// file system.
    name: String,

    /// The size of the file in bytes.
    ///
    /// This is the size of the file in bytes. The size is used to calculate
    /// the number of chunks that the file will be split into, and is also
    /// used to keep track of the progress of the file being sent.
    size: u64,
}

/// The context for the sender.
///
/// This structure contains all the information that the sender needs in order
/// to function properly. It is used to keep track of the state of the
/// sender, and to pass information between functions.
struct Context {
    /// The HMAC key for the sender.
    ///
    /// This is the key that is used to sign packets. The key is also used to
    /// generate a URL that the receiver can use to join the session.
    hmac: Vec<u8>,

    /// The sender that is used to send packets to the receiver.
    ///
    /// This sender is used to send handshake packets, list packets, chunk
    /// packets, and progress packets to the receiver.
    sender: Sender,

    /// The ephemeral keypair that is used to establish a shared key with the
    /// receiver.
    ///
    /// This key is used to establish a shared key between the sender and
    /// receiver. The key is ephemeral, meaning that it is only used once in
    /// the session. The key is generated when the sender is created, and is
    /// then discarded after the session is complete.
    key: EphemeralSecret,

    /// The files that the sender wants to send.
    ///
    /// This vec contains all the information about the files that the sender
    /// wants to send. The vec is filled when the user specifies the files to
    /// send using the command line arguments.
    files: Vec<File>,

    /// The shared key that is used to encrypt packets.
    ///
    /// This value is set to `None` initially, and is set to `Some` when the
    /// shared key is established with the receiver. The shared key is used to
    /// encrypt packets that are sent to the receiver.
    shared_key: Option<Aes128Gcm>,

    /// The task that is running in the background to send chunks of files to
    /// the receiver.
    ///
    /// This task is created when the sender is created, and is used to send
    /// chunks of files to the receiver in the background. The task is
    /// initially set to `None`, but is set to `Some` when the task is spawned.
    /// The task is used to cancel the background task when the sender is
    /// dropped.
    task: Option<JoinHandle<()>>,
}

/// This function is called when the client receives a create room packet
/// from the server. The function is responsible for printing a URL to the
/// console that the user can use to join the room.
///
/// The function first generates a base64 string from the hmac value that is
/// used to verify the integrity of the room. The base64 string is then
/// appended to the room id to create a URL. The URL is then printed to the
/// console using the qr2term library. Finally, the function prints a
/// message to the console with the URL.
fn on_create_room(context: &Context, id: String) -> Status {
    let base64 = general_purpose::STANDARD.encode(&context.hmac);
    let url = format!("{}-{}", id, base64);

    // Print a newline to the console to separate the output from the command
    // line.
    println!();

    // Try to generate a QR code from the URL. If the function fails for some
    // reason, print an error message to the console.
    if let Err(error) = qr2term::print_qr(&url) {
        error!("Failed to generate QR code: {}", error);
    }

    // Print a newline to the console to separate the output from the command
    // line.
    println!();

    // Print a message to the console with the URL.
    println!("Created room: {}", url);

    // Continue the event loop.
    Status::Continue()
}

/// This function is called when the client receives a join room packet from
/// the server. The function is responsible for sending a handshake packet to
/// the server containing the client's public key and a signature generated
/// using the client's private key and the room's hmac value.
///
/// The function first generates the client's public key from the private key.
/// The public key is then serialized into a byte array.
///
/// Next, the function creates a HMAC object with the room's hmac value and
/// updates it with the serialized public key. The resulting HMAC is then
/// serialized into a byte array and used as the signature in the handshake
/// packet.
///
/// Finally, the function sends the handshake packet to the server using the
/// sender object.
fn on_join_room(context: &Context, size: Option<usize>) -> Status {
    if size.is_some() {
        return Status::Err("Invalid join room packet.".into());
    }

    // Generate the client's public key from the private key.
    let public_key = context.key.public_key().to_sec1_bytes().into_vec();

    // Create a HMAC object with the room's hmac value and update
    // it with the serialized public key.
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();
    mac.update(&public_key);

    // Serialize the resulting HMAC into a byte array and use it as the
    // signature in the handshake packet.
    let signature = mac.finalize().into_bytes().to_vec();

    // Create the handshake packet and send it to the server.
    let handshake = HandshakePacket {
        public_key,
        signature,
    };

    context
        .sender
        .send_packet(DESTINATION, Value::Handshake(handshake));

    Status::Continue()
}

/// This function is called when an error packet is received from the
/// server. It creates a `Status::Err` variant containing the error
/// message from the server and returns it to be handled by the main
/// event loop.
///
/// When an error occurs, the server sends an error packet to the
/// client. The error packet contains a message with a description of
/// the error. This function extracts that message and creates a
/// `Status::Err` variant with it, which is then returned to be handled
/// by the main event loop.
///
/// The main event loop checks the status of the client and performs
/// the necessary actions based on its value. If the status is
/// `Status::Err`, the event loop exits with an error message
/// containing the error message from the server.
///
/// This function is called from the event loop when an error packet is
/// received from the server.
fn on_error(message: String) -> Status {
    Status::Err(message)
}

/// This function is called when the server sends a leave room packet to
/// the client. It is responsible for aborting the file transfer task,
/// generating a new ECDH key pair for the next handshake, and setting the
/// shared key to `None`.
///
/// When the server sends a leave room packet to the client, it means that
/// the receiver has disconnected from the room. In this case, the client
/// should abort the file transfer task and print an error message to the
/// user.
///
/// If the client is currently transferring files, it should abort the task
/// by calling `AbortHandle::abort` on the task handle.
///
/// After that, the client should generate a new ECDH key pair using the
/// `EphemeralSecret::random` function from the `p256` crate. This key pair
/// will be used for the next handshake with the server.
///
/// Finally, the client should set the shared key to `None` to indicate that
/// there is no shared key established for the current room.
///
/// This function is called from the event loop when a leave room packet is
/// received from the server.
fn on_leave_room(context: &mut Context, _: usize) -> Status {
    if let Some(task) = &context.task {
        // If the client is currently transferring files, abort the task
        // by calling `AbortHandle::abort` on the task handle.
        task.abort();
    }

    // Generate a new ECDH key pair for the next handshake.
    context.key = EphemeralSecret::random(&mut OsRng);

    // Set the shared key to `None` to indicate that there is no shared key
    // established for the current room.
    context.shared_key = None;

    // Set the task handle to `None` to indicate that there is no task
    // running.
    context.task = None;

    // Print an error message to the user indicating that the transfer was
    // interrupted because the receiver disconnected.
    println!();
    error!("Transfer was interrupted because the receiver disconnected.");

    // Continue the event loop.
    Status::Continue()
}

/// This function is called by the event loop when a progress packet is
/// received from the server.
///
/// The progress packet contains the index of the file that is being
/// transferred and the current progress of that file as a percentage.
///
/// If the client does not have a shared key established with the server,
/// the function returns an error and does not continue. This indicates
/// that the event loop should exit with an error message.
///
/// The function then retrieves the file at the index specified by the
/// progress packet from the context. If the index is out of bounds, the
/// function returns an error and does not continue. This indicates that
/// the event loop should exit with an error message.
///
/// The function then prints a message to the console indicating which file
/// is currently being transferred and what its progress is. The progress
/// message is printed to the same line as a carriage return (`\r`) so that
/// it overwrites the previous message.
///
/// If the progress of the file is 100%, the function prints a newline
/// (`\n`) to the console to move the cursor to the next line.
///
/// If the progress of the last file is 100%, the function returns
/// `Status::Exit()`. This indicates that the event loop should exit
/// successfully.
///
/// If any other condition is met, the function returns `Status::Continue()`.
/// This indicates that the event loop should continue running.
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

/// This function reads a file in chunks, sends each chunk to the receiver over
/// the WebSocket connection, and then sleeps for a short amount of time
/// before sending the next chunk.
///
/// The function takes the sender, the shared key, and a vector of files to
/// transfer as arguments.
///
/// For each file in the vector of files, the function reads the file in
/// chunks, sends each chunk to the receiver over the WebSocket connection,
/// and then sleeps for a short amount of time before sending the next chunk.
///
/// The chunk size is set to the maximum chunk size. If the number of bytes
/// left to read in the file is less than the chunk size, the chunk size is set
/// to the number of bytes left to read.
///
/// The function opens the file for reading using the tokio::fs::File::open
/// function. If there is an error opening the file, the function prints an
/// error message to the console and returns.
///
/// The function reads the file in chunks using the read_exact function from
/// the tokio::io::AsyncReadExt trait. If there is an error reading from the
/// file, the function prints an error message to the console and returns.
///
/// The function sends each chunk to the receiver over the WebSocket
/// connection using the send_encrypted_packet function from the Sender struct.
/// The function also increments the sequence number for each chunk that is
/// sent.
///
/// After sending all of the chunks for a file, the function sleeps for a short
/// amount of time using the tokio::time::sleep function. This helps to prevent
/// the sender from overwhelming the receiver with too many messages.
///
/// The function repeats this process for all of the files in the vector of
/// files.
async fn on_chunk(sender: Sender, shared_key: Option<Aes128Gcm>, files: Vec<File>) {
    for file in files {
        // Initialize a sequence number for the chunks of this file
        let mut sequence = 0;
        // Set the chunk size to the maximum chunk size
        let mut chunk_size = MAX_CHUNK_SIZE;
        // Set the number of bytes left to read in the file
        let mut size = file.size as isize;

        // Open the file for reading
        let mut handle = match tokio::fs::File::open(file.path).await {
            Ok(handle) => handle,
            Err(error) => {
                println!("Error: Unable to open file '{}': {}", file.name, error);
                return;
            }
        };

        while size > 0 {
            // If the number of bytes left to read in the file is less than the
            // chunk size, set the chunk size to the number of bytes left to read
            if size < chunk_size {
                chunk_size = size;
            }

            // Create a vector to hold the chunk of data to be read from the file
            let mut chunk = vec![0u8; chunk_size.try_into().unwrap()];
            // Read a chunk of data from the file into the vector
            handle.read_exact(&mut chunk).await.unwrap();

            // Send the chunk to the receiver over the WebSocket connection
            sender.send_encrypted_packet(
                &shared_key,
                DESTINATION,
                Value::Chunk(ChunkPacket { sequence, chunk }),
            );

            // Increment the sequence number for the next chunk
            sequence += 1;
            // Decrement the number of bytes left to read in the file
            size -= chunk_size;
        }

        // Sleep for a short amount of time to prevent overwhelming the receiver
        // with too many messages
        sleep(DELAY).await;
    }
}

/// This function sends a ListPacket to the receiver containing the list of
/// files to be transferred. The ListPacket contains a vector of Entry structs,
/// each of which represents one file.
///
/// The function creates a vector of Entry structs from the vector of File structs
/// in the Context struct. Each Entry struct contains the index, name, and size
/// of the corresponding File struct.
///
/// The function then sends the ListPacket to the receiver using the send_encrypted_packet
/// function from the Sender struct.
///
/// After sending the ListPacket, the function spawns a task using tokio::spawn to
/// call the on_chunk function with the Sender, shared_key, and vector of File
/// structs as arguments. The on_chunk function will send each chunk of data for
/// each file to the receiver.
///
/// The function returns Status::Continue(), which tells the main loop to continue
/// running until all of the files have been transferred.
fn on_handshake_finalize(context: &mut Context) -> Status {
    let mut entries = vec![];

    for (index, file) in context.files.iter().enumerate() {
        let entry = list_packet::Entry {
            // The index of the file in the vector of Files in the Context struct
            index: index.try_into().unwrap(),
            // The name of the file
            name: file.name.clone(),
            // The size of the file in bytes
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

/// Handshake function that is called when the Sender receives a HandshakeResponsePacket
/// from the Receiver. This function verifies the signature from the Receiver and if
/// successful, creates a shared key using the from the PublicKey struct.
///
/// The shared key is used to encrypt and decrypt packets sent between the Sender
/// and the Receiver.
///
/// This function is called by the main loop in client.rs.
fn on_handshake(context: &mut Context, handshake_response: HandshakeResponsePacket) -> Status {
    if context.shared_key.is_some() {
        // If the shared key is already established, this means that the Sender
        // has already performed the handshake, so return an error.
        return Status::Err("Already performed handshake.".into());
    }

    // Create a new HMAC using the hmac from the Context struct as the key.
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    // Update the HMAC with the public key from the HandshakeResponsePacket.
    mac.update(&handshake_response.public_key);

    // Call verify_slice() on the HMAC to verify the signature from the Receiver.
    // If the signature is invalid, return an error.
    let verification = mac.verify_slice(&handshake_response.signature);
    if verification.is_err() {
        return Status::Err("Invalid signature from the receiver.".into());
    }

    // Create a new PublicKey struct from the public key bytes in the
    // HandshakeResponsePacket.
    let shared_public_key = PublicKey::from_sec1_bytes(&handshake_response.public_key).unwrap();

    // Use the diffie_hellman() method from the PublicKey struct to create a shared
    // secret key between the Sender and the Receiver. The shared secret key is a
    // 16 byte long slice of bytes.
    let shared_secret = context.key.diffie_hellman(&shared_public_key);
    let shared_secret = shared_secret.raw_secret_bytes();
    let shared_secret = &shared_secret[0..16];

    // Create a new Key struct from the shared secret key. The Key<Aes128Gcm> type
    // is used to encrypt and decrypt packets.
    let shared_key: &Key<Aes128Gcm> = shared_secret.into();
    let shared_key = <Aes128Gcm as aes_gcm::KeyInit>::new(shared_key);

    // Set the shared_key field of the Context struct to the shared key.
    context.shared_key = Some(shared_key);

    // Call on_handshake_finalize() to start the transfer of files between the
    // Sender and the Receiver.
    on_handshake_finalize(context)
}

/// This function is called by the `Sender` when a new message is received over
/// the WebSocket connection. The message could be a text message or a binary
/// message. If it is a text message, it will be deserialized into a
/// `JsonPacketResponse` enum. If it is a binary message, it will be decrypted
/// if necessary and then deserialized into a `Packet` struct.
///
/// The `JsonPacketResponse` enum will have one of the following variants:
///
/// * `Create { id }`: The Receiver has created a new room with the given ID.
/// * `Join { size }`: The Receiver has joined a room with `size` number of
///   files.
/// * `Leave { index }`: The Receiver has left a room.
/// * `Error { message }`: The Receiver has encountered an error.
///
/// If the message is a binary message, the `Packet` struct will have a
/// `Value` variant that will have one of the following variants:
///
/// * `HandshakeResponse`: The Receiver has responded to the Sender's
///   `Handshake` packet.
/// * `Progress`: The Receiver has sent progress information for one of the
///   files in the room.
///
/// This function does the following:
///
/// * If the message is a text message, it is deserialized into a
///   `JsonPacketResponse` enum and then matched on to call the appropriate
///   function.
/// * If the message is a binary message, it is decrypted if necessary and then
///   deserialized into a `Packet` struct. The `Value` variant of the `Packet`
///   struct is then matched on to call the appropriate function.
///
/// If the message is invalid, an error is returned.
fn on_message(context: &mut Context, message: WebSocketMessage) -> Status {
    if message.is_text() {
        let text = message.into_text().unwrap();
        let packet = serde_json::from_str(&text).unwrap();

        return match packet {
            JsonPacketResponse::Create { id } => on_create_room(context, id),
            JsonPacketResponse::Join { size } => on_join_room(context, size),
            JsonPacketResponse::Leave { index } => on_leave_room(context, index),
            JsonPacketResponse::Error { message } => on_error(message),
        };
    } else if message.is_binary() {
        let data = message.into_data();
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
            Value::HandshakeResponse(handshake_response) => {
                on_handshake(context, handshake_response)
            }
            Value::Progress(progress) => on_progress(context, progress),

            _ => Status::Err(format!("Unexpected packet: {:?}", value)),
        };
    }

    Status::Err("Invalid message type".into())
}

/// Starts the sender client. This function will attempt to create a room with a size of 2
/// (the number of clients that will be joining the room) and then it will open a file for
/// each of the paths provided. It will then read chunks of data from each file and send them
/// to the server.
///
/// This function takes two arguments:
/// 1. `socket`: A `Socket` that represents the connection to the server.
/// 2. `paths`: A `Vec` of `String`s that represent the paths to the files that will be sent
///     to the server.
///
/// When the function is finished, it will exit and the transfer will be complete. If there
/// is an error during the transfer, the function will print an error message to stdout and
/// exit.
pub async fn start(socket: Socket, paths: Vec<String>) {
    // Create a vector to store metadata about each file that will be sent.
    let mut files = vec![];

    // For each path in the `paths` vector:
    for path in paths {
        // Attempt to open the file at the given path.
        let handle = match fs::File::open(&path) {
            // If the file is successfully opened, store it in the `handle` variable.
            Ok(handle) => handle,
            // If there is an error, print an error message to stdout and exit the function.
            Err(error) => {
                error!("Error: Failed to open file '{}': {}", path, error);
                return;
            }
        };

        // Get the metadata for the file.
        let metadata = handle.metadata().unwrap();

        // If the file is a directory, print an error message to stdout and exit the function.
        if metadata.is_dir() {
            error!("Error: The path '{}' does not point to a file.", path);
            return;
        }

        // Get the file name from the path.
        let name = Path::new(&path).file_name().unwrap().to_str().unwrap();

        // Get the file size from the metadata.
        let size = metadata.len();

        // If the file is empty, print an error message to stdout and exit the function.
        if size == 0 {
            error!("Error: The file '{}' is empty and cannot be sent.", name);
            return;
        }

        // Add the file metadata to the `files` vector.
        files.push(File {
            name: name.to_string(),
            path,
            size,
        });
    }

    // Generate a random key for HMAC.
    let mut hmac = [0u8; 32];
    OsRng.fill_bytes(&mut hmac);

    // Generate a random key for AES-GCM.
    let key = EphemeralSecret::random(&mut OsRng);

    // Create a channel to send packets to the server.
    let (sender, receiver) = flume::bounded(1000);

    // Split the socket into separate send and receive streams.
    let (outgoing, incoming) = socket.split();

    // Create a context that will be used throughout the transfer.
    let mut context = Context {
        // Store the sender half of the channel to send packets to the server.
        sender,
        // Store the ephemeral key for AES-GCM.
        key,
        // Store the files that will be sent to the server.
        files,

        // Store the HMAC key.
        hmac: hmac.to_vec(),
        // Set the shared key to None.
        shared_key: None,
        // Set the current task to None.
        task: None,
    };

    // Print a message to stdout indicating that the client is attempting to create a room.
    debug!("Attempting to create room...");

    // Send a JSON packet to the server to create a room with a size of 2.
    context.sender.send_json_packet(JsonPacket::Create);

    // Create a future that handles the outgoing stream of messages from the client to the
    // server.
    let outgoing_handler = receiver.stream().map(Ok).forward(outgoing);

    // Create a future that handles the incoming stream of messages from the server to the
    // client.
    let incoming_handler = incoming.try_for_each(|message| {
        // Call the `on_message` function to handle the incoming message.
        match on_message(&mut context, message) {
            // If the status is `Status::Exit`, the transfer is complete. Print a message to
            // stdout and exit the function.
            Status::Exit() => {
                // TODO: Signal Exit to the server
                println!("Transfer has completed.");

                // Exit the function with a `Result` of `Err`.
                return future::err(Error::ConnectionClosed);
            }
            // If the status is `Status::Err`, there was an error. Print an error message to
            // stdout and exit the function.
            Status::Err(error) => {
                error!("Error: {}", error);

                // Exit the function with a `Result` of `Err`.
                return future::err(Error::ConnectionClosed);
            }
            // Otherwise, the message was handled successfully.
            _ => {}
        };

        // Continue handling the incoming messages.
        future::ok(())
    });

    // Pin the `incoming_handler` and `outgoing_handler` futures so that they do not move.
    pin_mut!(incoming_handler, outgoing_handler);

    // Wait for either the `incoming_handler` or `outgoing_handler` to complete. If the
    // `incoming_handler` completes, return the result of the `incoming_handler`. If the
    // `outgoing_handler` completes, return the result of the `outgoing_handler`.
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
                    .to_string()
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
                WebSocketMessage::Text(r#"{"type":"leave","index":5}"#.to_string())
            ),
            Status::Continue()
        );
        assert_eq!(on_message(&mut context, WebSocketMessage::Text(r#"{"type":"create","id":"b531e87d-e51a-4507-94f4-335cbe2d32f3-Nc5skZReq7qJN7INwckyAZLWEEbxsrFfH/692tUNgkM="}"#.to_string())), Status::Continue());
        assert_eq!(
            on_message(
                &mut context,
                WebSocketMessage::Text(
                    r#"{"type":"error","message":"Error Message: Test"}"#.to_string()
                )
            ),
            Status::Err("Error Message: Test".to_string())
        );
    }
}
