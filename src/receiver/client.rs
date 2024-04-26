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

/// This struct represents a file that is being received.
///
/// The struct contains information about the file, such as its name, size,
/// and the handle of the file on the disk.
///
/// The `name` field contains the name of the file, which is the name of the
/// file on the disk.
///
/// The `size` field contains the size of the file in bytes.
///
/// The `progress` field contains the number of bytes that have already been
/// received for the file.
///
/// The `handle` field contains a handle to the file on the disk, which is
/// used to read the contents of the file.
struct File {
    name: String,
    size: u64,
    progress: u64,
    handle: fs::File,
}

/// This struct contains the context for the receiver.
///
/// This structure is used to keep track of the state of the receiver, and to
/// pass information between functions.
struct Context {
    /// The HMAC key that is used to verify that the packets that are received
    /// are authentic.
    ///
    /// The HMAC key is generated by the sender, and is used to verify that the
    /// packets that are received are authentic. If the HMAC of a packet does
    /// not match the expected HMAC, then the packet is not processed.
    hmac: Vec<u8>,

    /// The sender that is used to send packets to the server.
    ///
    /// The sender is used to send packets to the server. The sender is also
    /// used to receive packets from the server.
    sender: Sender,

    /// The ephemeral secret key that is used for key exchange with the sender.
    ///
    /// The ephemeral secret key is generated by the receiver, and is used to
    /// exchange a shared key with the sender. The shared key is used to
    /// encrypt and decrypt packets.
    key: EphemeralSecret,

    /// The shared key that is used to encrypt and decrypt packets.
    ///
    /// The shared key is established between the receiver and the sender during
    /// the key exchange. The shared key is used to encrypt and decrypt packets
    /// between the receiver and the sender. If the shared key is `None`, then the
    /// packets that are received are not processed.
    shared_key: Option<Aes128Gcm>,

    /// The files that are being received.
    ///
    /// The files vector contains a list of all the files that are being
    /// received. Each file is represented by a `File` struct. The `name` field
    /// of the `File` struct contains the name of the file, which is the name of
    /// the file on the disk. The `size` field of the `File` struct contains the
    /// size of the file in bytes. The `progress` field of the `File` struct
    /// contains the number of bytes that have already been received for the
    /// file. The `handle` field of the `File` struct contains a handle to the
    /// file on the disk, which is used to read the contents of the file.
    files: Vec<File>,

    /// The sequence number of the next chunk that is expected to be received.
    ///
    /// The sequence number is used to keep track of the sequence of chunks that
    /// are received. If a chunk does not have the expected sequence number,
    /// then the chunk is not processed.
    sequence: u32,

    /// The index of the file that is currently being received.
    ///
    /// The index is used to keep track of which file is currently being
    /// received. The index is incremented after a file is completely received.
    index: usize,

    /// The progress of the current file that is being received.
    ///
    /// The progress is used to keep track of the progress of the current file
    /// that is being received. The progress is calculated by dividing the
    /// number of bytes that have been received by the size of the file. The
    /// progress is sent to the server so that the sender knows how much of the
    /// file has been received.
    progress: u64,

    /// The total length of the current file that is being received.
    ///
    /// The length is used to keep track of the total length of the current file
    /// that is being received. The length is used to calculate the progress of
    /// the file.
    length: u64,
}

/// This function is called when the receiver receives a join room packet from
/// the server. The packet contains the size of the list of files that the
/// sender is going to send.
///
/// If the packet does not contain the size of the list, then an error is
/// returned.
///
/// If the packet does contain the size of the list, then a message is printed
/// to the console indicating that the receiver has connected to the room.
///
/// The function does not do anything else. It returns a `Status::Continue`
/// variant to indicate that the event loop should continue processing events.
fn on_join_room(size: Option<usize>) -> Status {
    // If the packet does not contain the size of the list, then return an error.
    if size.is_none() {
        return Status::Err("Invalid join room packet.".into());
    }

    // If the packet contains the size of the list, then print a message to the
    // console indicating that the receiver has connected to the room.
    println!("Connected to room.");

    // Return a `Status::Continue` variant to indicate that the event loop
    // should continue processing events.
    Status::Continue()
}

/// This function is called when the event loop receives an error packet from
/// the server. The packet contains a message with a description of the error.
///
/// When an error occurs, the server sends an error packet to the client. The
/// error packet contains a message with a description of the error. This
/// function extracts that message and creates a `Status::Err` variant with it,
/// which is then returned to be handled by the main event loop.
///
/// When the event loop receives a status variant that is an error, it exits
/// with an error message containing the message from the error packet.
///
/// The message from the error packet is the only information that the event
/// loop has about the error, so the message should be descriptive and
/// helpful to the user. The message should not contain technical details
/// about the error or how it occurred. Instead, the message should be
/// written from the perspective of the user and should give the user enough
/// information to understand what went wrong and how they might be able to
/// fix the problem.
///
/// This function takes the message from the error packet and creates a
/// `Status::Err` variant with it. The function returns this variant to be
/// handled by the main event loop.
fn on_error(message: String) -> Status {
    Status::Err(message)
}

/// This function is called when the event loop receives a leave room packet from
/// the server. The packet contains the index of the file that was being
/// transferred when the receiver left the room.
///
/// When the receiver receives a leave room packet, it means that the sender
/// has disconnected from the room. In this case, the receiver should check if
/// there are any files that were being transferred but not yet complete. If
/// there are any incomplete files, the receiver should print a message to
/// the user indicating that the transfer was interrupted.
///
/// If there are no incomplete files, then the receiver should exit
/// normally. The `Status::Exit` variant is returned to the main event loop,
/// which will cause the event loop to exit normally.
///
/// This function checks if there are any incomplete files by iterating over
/// the list of files in the context. If there are any files with a progress
/// less than 100%, then the function prints a message to the user and returns
/// an error status.
///
/// If there are no incomplete files, then the function simply returns a
/// `Status::Exit` variant. This will cause the main event loop to exit
/// normally.
fn on_leave_room(context: &mut Context, _: usize) -> Status {
    // Check if there are any incomplete files.
    if context.files.iter().any(|file| file.progress < 100) {
        // If there are any incomplete files, print a message to the user.
        println!();
        println!("Transfer was interrupted because the host left the room.");

        // Return an error status.
        Status::Err("Transfer was interrupted because the host left the room.".into())
    } else {
        // If there are no incomplete files, return a `Status::Exit` variant.
        // This will cause the event loop to exit normally.
        Status::Exit()
    }
}

/// This function is called when the event loop receives a list packet from
/// the server. The packet contains a list of files to be transferred.
///
/// When this function is called, we know that the sender has successfully
/// established a shared key with the receiver. Therefore, we can start
/// receiving encrypted files.
///
/// This function iterates over the list of files in the packet and creates a
/// file on disk for each file in the list. If a file with the same name already
/// exists, an error is returned and the event loop is exited with an error
/// message. This is because the receiver should not overwrite existing files
/// without the user's explicit permission.
///
/// Once all the files have been created, the function initializes the context
/// variables for the event loop. `index` is set to 0 to indicate that we are
/// currently transferring the first file. `progress` is set to 0 to indicate
/// that the progress of the first file is 0%. `sequence` is set to 0 to
/// indicate that the first chunk of data we receive will have a sequence
/// number of 0. `length` is set to 0 to indicate that we have not received
/// any data yet.
///
/// If there is an error creating any of the files, the function returns an
/// error status. This will cause the event loop to exit with an error message.
///
/// If there are no errors, the function returns a `Status::Continue()` variant.
/// This will cause the event loop to continue running and wait for more
/// packets from the sender.
fn on_list(context: &mut Context, list: ListPacket) -> Status {
    if context.shared_key.is_none() {
        return Status::Err("Invalid list packet: no shared key established".into());
    }

    // Iterate over the list of files in the packet.
    for entry in list.entries {
        // Sanitize the file name to remove any characters that are not valid in
        // file names on the current platform.
        let path = sanitize_filename::sanitize(entry.name.clone());

        // Check if a file with the same name already exists.
        if Path::new(&path).exists() {
            // If the file already exists, return an error and exit the event loop
            // with an error message.
            return Status::Err(format!("The file '{}' already exists.", path));
        }

        // Try to create a new file with the sanitized file name.
        let handle = match fs::File::create(&path) {
            Ok(handle) => handle,
            Err(error) => {
                // If there is an error creating the file, return an error and
                // exit the event loop with an error message.
                return Status::Err(format!(
                    "Error: Failed to create file '{}': {}",
                    path, error
                ));
            }
        };

        // Create a new file struct for the file we just created.
        let file = File {
            name: entry.name,
            size: entry.size,
            handle,
            progress: 0,
        };

        // Add the new file to the list of files in the context.
        context.files.push(file);
    }

    // Set the context variables for the event loop.
    context.index = 0;
    context.progress = 0;
    context.sequence = 0;
    context.length = 0;

    // Return a `Status::Continue()` variant to indicate that the event loop
    // should continue running and wait for more packets from the sender.
    Status::Continue()
}

/// This function handles a chunk packet received from the sender.
///
/// It checks that the shared key has been established, that the sequence number
/// of the chunk matches the expected sequence number in the context, and that
/// the index of the file in the context is valid.
///
/// If any of these checks fail, an error is returned and the event loop is
/// stopped.
///
/// The function updates the length of the file, increments the sequence number
/// in the context, and writes the contents of the chunk to the file.
///
/// The progress of the file is updated to be the ratio of the number of bytes
/// read so far to the total size of the file.
///
/// If the progress of the file is 100%, or if the difference in progress between
/// this chunk and the last chunk is greater than or equal to 1, or if this is the
/// first chunk, a ProgressPacket is sent to the sender with the index of the file
/// in the context and the progress of the file.
///
/// If the size of the file has been reached, the index of the current file is
/// incremented, the length of the current file is set to 0, the progress of the
/// current file is set to 0, and the sequence number is set to 0.
///
/// Finally, a Status::Continue() variant is returned to indicate that the event
/// loop should continue running and wait for more packets from the sender.
fn on_chunk(context: &mut Context, chunk: ChunkPacket) -> Status {
    // Check that the shared key has been established.
    if context.shared_key.is_none() {
        return Status::Err("Invalid chunk packet: no shared key established".into());
    }

    // Check that the sequence number of the chunk matches the expected sequence
    // number in the context.
    if chunk.sequence != context.sequence {
        return Status::Err(format!(
            "Expected sequence {}, but got {}.",
            context.sequence, chunk.sequence
        ));
    }

    // Get a mutable reference to the file in the context at the index of the
    // file.
    let Some(file) = context.files.get_mut(context.index) else {
        // If the index of the file in the context is invalid, return an error and
        // stop the event loop.
        return Status::Err("Invalid file index.".into());
    };

    // Update the length of the file.
    context.length += chunk.chunk.len() as u64;

    // Increment the sequence number in the context.
    context.sequence += 1;

    // Write the contents of the chunk to the file.
    file.handle.write(&chunk.chunk).unwrap();

    // Update the progress of the file.
    file.progress = (context.length * 100) / file.size;

    // If the progress of the file is 100%, or if the difference in progress between
    // this chunk and the last chunk is greater than or equal to 1, or if this is the
    // first chunk, send a ProgressPacket to the sender.
    if file.progress == 100 || file.progress - context.progress >= 1 || chunk.sequence == 0 {
        context.progress = file.progress;

        let progress = ProgressPacket {
            // Convert the index of the file in the context to a u32.
            index: context.index.try_into().unwrap(),
            // Convert the progress of the file to a u32.
            progress: context.progress.try_into().unwrap(),
        };

        // Send the ProgressPacket to the sender.
        context.sender.send_encrypted_packet(
            &context.shared_key,
            DESTINATION,
            Value::Progress(progress),
        );

        print!("\rTransferring '{}': {}%", file.name, file.progress);
        std::io::Write::flush(&mut stdout()).unwrap();
    }

    // If the size of the file has been reached, increment the index of the
    // current file, set the length of the current file to 0, set the progress
    // of the current file to 0, and resets the sequence number to 0.
    if file.size == context.length {
        context.index += 1;
        context.length = 0;
        context.progress = 0;
        context.sequence = 0;

        println!();
    }

    // Return a Status::Continue() variant to indicate that the event loop should
    // continue running and wait for more packets from the sender.
    Status::Continue()
}

/// This function is called when the Receiver receives a HandshakePacket from the
/// Sender. It verifies the signature of the Sender's public key and generates its own
/// public key. It then generates a shared secret key between the Receiver and the Sender
/// using the Diffie-Hellman key exchange.
///
/// The Receiver sends back a HandshakeResponsePacket to the Sender with its own public
/// key and a signature created using the shared secret key and its own private key.
///
/// The shared secret key is used to encrypt packets sent between the Receiver and the
/// Sender.
fn on_handshake(context: &mut Context, handshake: HandshakePacket) -> Status {
    // If a shared key has already been established, this means that the Receiver
    // has already performed the handshake, so return an error.
    if context.shared_key.is_some() {
        return Status::Err("Already performed handshake.".into());
    }

    // Create a new HMAC using the hmac from the Context struct as the key.
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    // Update the HMAC with the public key from the HandshakePacket.
    mac.update(&handshake.public_key);

    // Call verify_slice() on the HMAC to verify the signature from the Sender.
    // If the signature is invalid, return an error.
    let verification = mac.verify_slice(&handshake.signature);
    if verification.is_err() {
        return Status::Err("Invalid signature from the sender.".into());
    }

    // Generate the Receiver's public key from the private key.
    let public_key = context.key.public_key().to_sec1_bytes().into_vec();

    // Create a new HMAC using the hmac from the Context struct as the key.
    let mut mac = Hmac::<Sha256>::new_from_slice(&context.hmac).unwrap();

    // Update the HMAC with the public key of the Receiver.
    mac.update(&public_key);

    // Serialize the resulting HMAC into a byte array and use it as the
    // signature in the HandshakeResponsePacket.
    let signature = mac.finalize().into_bytes().to_vec();
    // Create a new shared secret key between the Receiver and the Sender.
    let shared_public_key = PublicKey::from_sec1_bytes(&handshake.public_key).unwrap();

    let shared_secret = context.key.diffie_hellman(&shared_public_key);
    let shared_secret = shared_secret.raw_secret_bytes();
    let shared_secret = &shared_secret[0..16];

    // Create a new Aes128Gcm key from the shared secret.
    let shared_key: &Key<Aes128Gcm> = shared_secret.into();
    let shared_key = <Aes128Gcm as aes_gcm::KeyInit>::new(shared_key);

    // Create the HandshakeResponsePacket and send it to the Sender.
    let handshake_response = HandshakeResponsePacket {
        public_key,
        signature,
    };

    context
        .sender
        .send_packet(DESTINATION, Value::HandshakeResponse(handshake_response));

    // Store the shared key in the Context struct.
    context.shared_key = Some(shared_key);

    // Return a Status::Continue() variant to indicate that the event loop should
    // continue running and wait for more packets from the Sender.
    Status::Continue()
}

/// This function is called when a message is received from the Sender.
///
/// The message can be either text or binary. If it's text, we attempt to
/// parse it as a JsonPacketResponse and match on the type of response it is.
/// If it's binary, we attempt to decrypt it using the shared key (if it
/// exists) and then decode it into a Packet. We then match on the type of
/// value in the Packet and call the appropriate function with the relevant
/// data.
///
/// If the message is not text or binary, we return a Status::Err with an
/// appropriate error message.
fn on_message(context: &mut Context, message: WebSocketMessage) -> Status {
    if message.is_text() {
        let text = message.into_text().unwrap();
        let packet = serde_json::from_str(&text).unwrap();

        return match packet {
            JsonPacketResponse::Join { size } => on_join_room(size),
            JsonPacketResponse::Leave { index } => on_leave_room(context, index),
            JsonPacketResponse::Error { message } => on_error(message),

            _ => Status::Err(format!("Unexpected json packet: {:?}", packet)),
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
            Value::List(list) => on_list(context, list),
            Value::Chunk(chunk) => on_chunk(context, chunk),
            Value::Handshake(handshake) => on_handshake(context, handshake),

            _ => Status::Err(format!("Unexpected packet: {:?}", value)),
        };
    }

    Status::Err("Invalid message type".into())
}

/// This function takes a websocket connection and an invite code,
/// splits the connection into an outgoing and incoming part,
/// creates a context for the connection, sends a join room packet,
/// and starts two futures to handle incoming and outgoing messages.
///
/// The outgoing future reads from a channel and sends the messages
/// through the outgoing part of the connection. If the sending fails,
/// the future will print an error and exit.
///
/// The incoming future reads from the incoming part of the connection
/// and passes the messages to the `on_message` function. If the message
/// is an exit or an error, the function will print the error and exit.
/// If the message is any other type of packet, it will be handled by the
/// `on_message` function and the future will continue running.
pub async fn start(socket: Socket, fragment: &str) {
    // Extract the room id and hmac from the invite code
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

    // Create a new ephemeral key pair
    let key = EphemeralSecret::random(&mut OsRng);

    // Create a channel for sending messages
    let (sender, receiver) = flume::bounded(1000);

    // Split the websocket connection into an outgoing and incoming part
    let (outgoing, incoming) = socket.split();

    // Create a new context for the connection
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

    // Send a join room packet to the server
    context
        .sender
        .send_json_packet(JsonPacket::Join { id: id.to_string() });

    // Create futures for handling incoming and outgoing messages
    let outgoing_handler = receiver.stream().map(Ok).forward(outgoing);
    let incoming_handler = incoming.try_for_each(|message| {
        // Call the on_message function to handle the message
        match on_message(&mut context, message) {
            // If the message is an exit, print a message and exit
            Status::Exit() => {
                println!("Transfer has completed.");

                return future::err(Error::ConnectionClosed);
            }
            // If the message is an error, print the error and exit
            Status::Err(error) => {
                println!("Error: {}", error);

                return future::err(Error::ConnectionClosed);
            }
            // If the message is any other type of packet, do nothing
            _ => {}
        };

        // Continue running the future
        future::ok(())
    });

    // Pin the futures to the stack so they can run concurrently
    pin_mut!(incoming_handler, outgoing_handler);

    // Wait for either future to complete
    future::select(incoming_handler, outgoing_handler).await;
}
