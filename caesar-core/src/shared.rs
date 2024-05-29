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

/// Represents a packet that is sent over a websocket connection.
///
/// This enum is used to represent different types of packets that can be sent over a websocket connection.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JsonPacket {
    /// A packet to join a room.
    ///
    /// This variant is used to request to join a room. The `id` field is used to specify the room id.
    Join {
        /// The id of the room to join.
        id: String,
    },
    /// A packet to create a new room.
    ///
    /// This variant is used to request to create a new room. The `id` field is used to specify the room id, which can be optional.
    Create {
        /// The id of the room to create. It can be `None` to generate a random room id.
        id: Option<String>,
    },
    /// A packet to leave a room.
    ///
    /// This variant is used to request to leave a room.
    Leave,
}

/// Represents a response to a `JsonPacket` packet.
///
/// This enum is used to represent different types of responses to a `JsonPacket` packet.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JsonPacketResponse {
    /// A response to a `Join` packet.
    ///
    /// This variant is used to indicate the result of a `Join` packet. The `size` field is used to specify the number of existing users in the room.
    Join {
        /// The number of existing users in the room. This field is `None` if the room is empty.
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<usize>,
    },
    /// A response to a `Create` packet.
    ///
    /// This variant is used to indicate the result of a `Create` packet. The `id` field is used to specify the room id.
    Create {
        /// The id of the created room.
        id: String,
    },
    /// A response to a `Leave` packet.
    ///
    /// This variant is used to indicate the result of a `Leave` packet. The `index` field is used to specify the index of the user who left the room.
    Leave {
        /// The index of the user who left the room.
        index: usize,
    },
    /// An error response.
    ///
    /// This variant is used to indicate an error. The `message` field is used to specify the error message.
    Error {
        /// The error message.
        message: String,
    },
}

/// Represents the result of an operation.
///
/// This enum is used to indicate the status of an operation. It can be one of three
/// variants:
///
/// - `Continue`: Operation was successful and the client should continue.
/// - `Exit`: Operation was successful and the client should exit.
/// - `Err`: Operation encountered an error. The error message is provided in the
///   variant.
#[derive(Debug, PartialEq)]
pub enum Status {
    /// Operation was successful and the client should continue.
    Continue(),
    /// Operation was successful and the client should exit.
    Exit(),
    /// Operation encountered an error. The error message is provided in the variant.
    Err(String),
}

/// Represents a sender of JSON packets.
///
/// This trait is used to send JSON packets to a `JsonPacket` receiver. The
/// `send_json_packet` method is used to send a `JsonPacket` packet.
pub trait JsonPacketSender {
    /// Sends a `JsonPacket` packet to a receiver.
    ///
    /// This method sends a `JsonPacket` packet to a receiver. The `packet` argument
    /// is the packet to send.
    fn send_json_packet(&self, packet: JsonPacket);
}

/// Represents a sender of packets.
///
/// This trait is used to send packets to a receiver. The `send_packet` method is used to send
/// a plain packet, and the `send_encrypted_packet` method is used to send an encrypted packet.
pub trait PacketSender {
    /// Sends a plain packet to a receiver.
    ///
    /// This method sends a plain packet to a receiver. The `destination` argument specifies the
    /// destination of the packet, and the `packet` argument is the packet to send.
    fn send_packet(&self, destination: u8, packet: packets::packet::Value);

    /// Sends an encrypted packet to a receiver.
    ///
    /// This method sends an encrypted packet to a receiver. The `key` argument is the encryption
    /// key to use, the `destination` argument specifies the destination of the packet, and the
    /// `value` argument is the packet to send.
    fn send_encrypted_packet(
        &self,
        key: &Option<Aes128Gcm>,
        destination: u8,
        value: packets::packet::Value,
    );
}


/// Implementation of `JsonPacketSender` for `Sender` struct.
///
/// This implementation of `JsonPacketSender` for `Sender` struct provides a method
/// `send_json_packet` to send a `JsonPacket` packet.
impl JsonPacketSender for Sender {
    /// Sends a `JsonPacket` packet to a receiver.
    ///
    /// This method serializes the `JsonPacket` using `serde_json` and sends it as a
    /// `WebSocketMessage::Text` to a receiver.
    ///
    /// # Arguments
    ///
    /// * `packet` - The `JsonPacket` to send.
    fn send_json_packet(&self, packet: JsonPacket) {
        // Serialize the JsonPacket using serde_json
        let serialized_packet = serde_json::to_string(&packet)
            .expect("Failed to serialize JSON packet.");

        // Send the serialized packet as a WebSocketMessage::Text
        self.send(WebSocketMessage::Text(serialized_packet))
            .expect("Failed to send JSON packet.");
    }
}

/// Implementation of `PacketSender` for `Sender` struct.
///
/// This implementation of `PacketSender` for `Sender` struct provides methods
/// to send a packet to a receiver.
impl PacketSender for Sender {
    /// Sends a packet to a receiver.
    ///
    /// This method serializes the packet and sends it as a `WebSocketMessage::Binary` to a receiver.
    ///
    /// # Arguments
    ///
    /// * `destination` - The destination of the packet.
    /// * `value` - The packet to send.
    fn send_packet(&self, destination: u8, value: packets::packet::Value) {
        // Serialize the packet
        let packet = Packet { value: Some(value) };
        let mut serialized_packet = packet.encode_to_vec();

        // Insert the destination at the beginning of the packet
        serialized_packet.insert(0, destination);

        // Send the serialized packet as a WebSocketMessage::Binary
        self.send(WebSocketMessage::Binary(serialized_packet))
            .expect("Failed to send Packet.");
    }

    /// Sends an encrypted packet to a receiver.
    ///
    /// This method encrypts the packet using the provided key and sends it as a
    /// `WebSocketMessage::Binary` to a receiver.
    ///
    /// # Arguments
    ///
    /// * `key` - The encryption key to use.
    /// * `destination` - The destination of the packet.
    /// * `value` - The packet to send.
    fn send_encrypted_packet(
        &self,
        key: &Option<Aes128Gcm>,
        destination: u8,
        value: packets::packet::Value,
    ) {
        // Serialize the packet
        let packet = Packet { value: Some(value) };

        // Generate a nonce for encryption
        let nonce = Aes128Gcm::generate_nonce(&mut OsRng);

        // Serialize the packet
        let plaintext = packet.encode_to_vec();

        // Encrypt the packet using the provided key
        let mut ciphertext = key
            .as_ref()
            .unwrap()
            .encrypt(&nonce, plaintext.as_ref())
            .expect("Failed to encrypt Packet.");

        // Create the serialized packet by concatenating the nonce and the ciphertext
        let mut serialized_packet = nonce.to_vec();
        serialized_packet.append(&mut ciphertext);

        // Insert the destination at the beginning of the packet
        serialized_packet.insert(0, destination);

        // Send the serialized packet as a WebSocketMessage::Binary
        self.send(WebSocketMessage::Binary(serialized_packet))
            .expect("Failed to send encrypted Packet.");
    }
}

pub type Sender = flume::Sender<WebSocketMessage>;

pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
