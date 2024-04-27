pub mod packets {
    include!(concat!(env!("OUT_DIR"), "/packets.rs"));
}

use aes_gcm::{
    aead::{Aead, AeadCore},
    Aes128Gcm,
};
use packets::Packet;
use prost::Message;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// This struct is used to serialize/deserialize JSON packets sent
/// between the client and the server.
///
/// The `type` field is used to specify the type of packet that is being sent.
/// The possible values for this field are listed as variants of the enum.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JsonPacket {
    /// Sent from the client to ask to join a room.
    ///
    /// The `id` field specifies the ID of the room that the client wants
    /// to join.
    Join {
        /// The ID of the room that the client wants to join.
        id: String,
    },
    /// Sent from the client to ask to create a new room.
    Create,
    /// Sent from the client to ask to leave the current room.
    Leave,
}

/// This struct is used to serialize/deserialize JSON packets sent
/// from the server to the client.
///
/// The `type` field is used to specify the type of packet that is being
/// sent. The possible values for this field are listed as variants of the
/// enum.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JsonPacketResponse {
    /// Sent from the server to inform the client of the result of a `Join`
    /// packet.
    ///
    /// If the client successfully joined a room, the `size` field will be
    /// `Some` and contain the size of the room. If the client could not join
    /// a room, the `size` field will be `None`.
    Join {
        /// The size of the room that the client joined. If the client could
        /// not join a room, this field will be `None`.
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<usize>,
    },
    /// Sent from the server to inform the client of the result of a `Create`
    /// packet.
    ///
    /// If the server successfully created a room, the `id` field will
    /// contain the ID of the room. If the server could not create a room,
    /// the `id` field will be empty.
    Create {
        /// The ID of the room that the server created. If the server could
        /// not create a room, this field will be empty.
        id: String,
    },
    /// Sent from the server to inform the client of the result of a `Leave`
    /// packet.
    ///
    /// If the client successfully left a room, the `index` field will
    /// contain the index of the client that left the room. If the client
    /// could not leave a room, the `index` field will be 0.
    Leave {
        /// The index of the client that left the room. If the client could
        /// not leave a room, this field will be 0.
        index: usize,
    },
    /// Sent from the server to inform the client of an error.
    ///
    /// The `message` field contains a description of the error.
    Error {
        /// A description of the error that occurred.
        message: String,
    },
}

/// This enum represents the result of processing an event in the event loop.
///
/// The `Status` enum has three variants:
///
/// * `Continue` - This variant indicates that the event loop should
///   continue processing events. This is the most common result and is used
///   when the event loop has nothing special to do.
///
/// * `Exit` - This variant indicates that the event loop should exit. This
///   is used when the event loop should exit because of an error or
///   because the user has requested that the program exit.
///
/// * `Err` - This variant indicates that the event loop encountered an
///   error. When the event loop receives a `Status::Err` variant, it should
///   exit with an error message containing the message from the error packet.
///   The message from the error packet is the only information that the event
///   loop has about the error, so the message should be descriptive and
///   helpful to the user. The message should not contain technical details
///   about the error or how it occurred. Instead, the message should be
///   written from the perspective of the user and should give the user enough
///   information to understand what went wrong and how they might be able to
///   fix the problem.
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Indicates that the event loop should continue processing events.
    Continue(),
    /// Indicates that the event loop should exit.
    Exit(),
    /// Indicates that the event loop encountered an error.
    Err(String),
}

/// A trait for sending JSON packets.
///
/// This trait provides a single method, `send_json_packet`, which sends a
/// JSON packet over some underlying transport.
pub trait JsonPacketSender {
    /// Sends a JSON packet.
    ///
    /// This method takes a single argument, `packet`, which is the JSON packet
    /// to send. The packet will be serialized into a JSON string and then sent
    /// over the underlying transport.
    ///
    /// Note that the exact semantics of what it means to "send a JSON packet"
    /// will depend on the specific implementation of this trait. However, in
    /// general, the packet will be sent as a single message over the
    /// transport, and the transport will be responsible for ensuring that the
    /// packet is delivered to the intended recipient.
    ///
    /// # Errors
    ///
    /// If there is an error serializing the JSON packet, or if there is an
    /// error sending the serialized packet over the transport, this method
    /// may return an error. The exact semantics of what constitutes an error
    /// will depend on the specific implementation of this trait.
    fn send_json_packet(&self, packet: JsonPacket);
}

/// A trait for sending Protocol Buffers packets over some underlying transport.
///
/// This trait provides two methods for sending Protocol Buffers packets:
///
/// * `send_packet` sends a packet in the clear (i.e., not encrypted).
/// * `send_encrypted_packet` sends a packet encrypted using the AES-GCM
///   algorithm with a 128-bit key.
///
/// The exact semantics of what it means to "send a packet" will depend on the
/// specific implementation of this trait. However, in general, the packet will
/// be serialized into a binary message using the Protocol Buffers wire format,
/// and then sent over the underlying transport.
///
/// The `destination` argument specifies which recipient should receive the
/// packet. This is a 1-byte field that is prepended to the serialized packet
/// before it is sent.
///
/// The `key` argument is an optional AES-GCM key. If a key is provided, the
/// packet will be encrypted before being sent. If no key is provided, the
/// packet will be sent in the clear.
///
/// # Errors
///
/// If there is an error serializing the Protocol Buffers packet, or if there
/// is an error sending the serialized packet over the transport, either of
/// these methods may return an error. The exact semantics of what constitutes
/// an error will depend on the specific implementation of this trait.
pub trait PacketSender {
    /// Sends a Protocol Buffers packet in the clear.
    ///
    /// The packet will be serialized into a binary message using the Protocol
    /// Buffers wire format, and then sent over the underlying transport.
    fn send_packet(&self, destination: u8, packet: packets::packet::Value);

    /// Sends a Protocol Buffers packet encrypted using AES-GCM.
    ///
    /// The packet will be serialized into a binary message using the Protocol
    /// Buffers wire format, encrypted using AES-GCM with a 128-bit key, and
    /// then sent over the underlying transport.
    ///
    /// If no key is provided, the packet will be sent in the clear.
    fn send_encrypted_packet(
        &self,
        key: &Option<Aes128Gcm>,
        destination: u8,
        value: packets::packet::Value,
    );
}

impl JsonPacketSender for Sender {
    /// Serializes the given JSON packet into a string, and then sends it as a
    /// text message over the underlying transport.
    ///
    /// The `JsonPacket` type is defined in the `serde_json` crate, and it is a
    /// simple wrapper around a JSON object with string keys and values. This
    /// trait method is responsible for taking a `JsonPacket` and sending it
    /// over the WebSocket connection.
    ///
    /// The `serde_json::to_string` function is used to serialize the packet
    /// into a JSON string. If this function returns an error, we panic
    /// because there is no reasonable recovery behavior in this case.
    ///
    /// Once we have the JSON string, we wrap it in a `WebSocketMessage::Text`
    /// enum variant and send it over the WebSocket connection using the
    /// `send` method. If this method returns an error, we panic because there
    /// is no reasonable recovery behavior in this case.
    fn send_json_packet(&self, packet: JsonPacket) {
        let serialized_packet =
            serde_json::to_string(&packet).expect("Failed to serialize JSON packet.");

        self.send(WebSocketMessage::Text(serialized_packet))
            .expect("Failed to send JSON packet.");
    }
}

impl PacketSender for Sender {
    /// Serializes the given packet value into a binary message, and then
    /// sends it over the underlying transport.
    ///
    /// The `destination` parameter specifies which client should receive
    /// this message. The value of this parameter should be a byte that
    /// represents the client's index in the list of connected clients.
    ///
    /// The `value` parameter specifies the actual data that should be sent
    /// to the client. This will be serialized into a `Packet` struct using
    /// the Protocol Buffers wire format.
    ///
    /// This function will first encode the `Packet` struct into a vector of
    /// bytes using the Protocol Buffers wire format. It will then insert the
    /// `destination` byte as the first element of the vector, so that the
    /// receiving client knows which client this message is intended for.
    ///
    /// Finally, this function will send the serialized packet over the
    /// underlying transport, which is assumed to be a WebSocket connection.
    /// If this send operation fails, this function will panic because there
    /// is no reasonable recovery behavior in this case.
    fn send_packet(&self, destination: u8, value: packets::packet::Value) {
        let packet = Packet { value: Some(value) };

        let mut serialized_packet = packet.encode_to_vec();
        serialized_packet.insert(0, destination);

        self.send(WebSocketMessage::Binary(serialized_packet))
            .expect("Failed to send Packet.");
    }

    /// Similar to `send_packet`, but the message is encrypted using AES-GCM
    /// with a 128-bit key.
    ///
    /// If no key is provided (i.e., if `key` is `None`), then the message will
    /// be sent in the clear.
    ///
    /// This function works by generating a random 12-byte nonce using the
    /// `rand::OsRng` PRNG, encrypting the message using AES-GCM with the
    /// provided key and nonce, and then prepending the nonce to the ciphertext
    /// before sending it over the WebSocket connection. The receiving client
    /// will use the same key and nonce to decrypt the message.
    ///
    /// Note that this function does not actually check whether the provided
    /// key is valid. If an invalid key is provided, the encryption will fail
    /// and the receiver will not be able to decrypt the message.
    fn send_encrypted_packet(
        &self,
        key: &Option<Aes128Gcm>,
        destination: u8,
        value: packets::packet::Value,
    ) {
        let packet = Packet { value: Some(value) };

        let nonce = Aes128Gcm::generate_nonce(&mut OsRng);
        let plaintext = packet.encode_to_vec();
        let mut ciphertext = key
            .as_ref()
            .unwrap()
            .encrypt(&nonce, plaintext.as_ref())
            .expect("Failed to encrypt Packet.");

        let mut serialized_packet = nonce.to_vec();
        serialized_packet.append(&mut ciphertext);
        serialized_packet.insert(0, destination);

        self.send(WebSocketMessage::Binary(serialized_packet))
            .expect("Failed to send encrypted Packet.");
    }
}

/// A sender is a type that allows us to send messages to a WebSocket client.
///
/// In this case, a sender is a channel that allows us to send WebSocket
/// messages to a client. The messages can be any type that implements the
/// `Into<WebSocketMessage>`.
///
/// The `WebSocketMessage` type represents any message that can be sent over a
/// WebSocket connection. It can be a binary message, a text message, or a
/// close message.
///
/// The `MaybeTlsStream` type is a stream that may or may not be encrypted.
/// If the connection is encrypted (e.g., via TLS), then the stream will be
/// encrypted. If the connection is not encrypted, then the stream will be
/// unencrypted.
///
/// The `TcpStream` type is a stream that is used to connect to a remote
/// server over a TCP connection.
///
/// The `WebSocketStream` type is a stream that is used to connect to a remote
/// WebSocket server. It is a wrapper around the `MaybeTlsStream` stream that
/// adds WebSocket-specific functionality.
pub type Sender = flume::Sender<WebSocketMessage>;

/// A socket is a type that represents a WebSocket connection.
///
/// In this case, a socket is a wrapper around a `MaybeTlsStream` stream that
/// adds WebSocket-specific functionality.
///
/// The `MaybeTlsStream` type is a stream that may or may not be encrypted.
/// If the connection is encrypted (e.g., via TLS), then the stream will be
/// encrypted. If the connection is not encrypted, then the stream will be
/// unencrypted.
///
/// The `TcpStream` type is a stream that is used to connect to a remote
/// server over a TCP connection.
///
/// The `WebSocketStream` type is a stream that is used to connect to a remote
/// WebSocket server. It is a wrapper around the `MaybeTlsStream` stream that
/// adds WebSocket-specific functionality.
pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
