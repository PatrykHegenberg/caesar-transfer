pub mod appstate;
pub mod client;
pub mod room;
pub mod server;
pub mod transfer;

use serde::{Deserialize, Serialize};


/// Represents a packet sent by a client to the server.
/// 
/// The `type` field is used to determine the type of the packet. It can be one of the following
/// values:
/// - `Join`: The client wants to join a room.
/// - `Create`: The client wants to create a new room.
/// - `Leave`: The client wants to leave the current room.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RequestPacket {
    /// The client wants to join a room.
    /// 
    /// The `id` field is the ID of the room that the client wants to join.
    Join {
        /// The ID of the room that the client wants to join.
        id: String,
    },
    /// The client wants to create a new room.
    /// 
    /// The `id` field is an optional field that specifies the ID of the new room. If it is `None`,
    /// a random ID will be generated.
    Create {
        /// The ID of the new room. If it is `None`, a random ID will be generated.
        id: Option<String>,
    },
    /// The client wants to leave the current room.
    Leave,
}

/// Represents a packet sent by the server to the client.
/// 
/// The `type` field is used to determine the type of the packet. It can be one of the following
/// values:
/// - `Join`: The client has joined a room.
/// - `Create`: The client has created a new room.
/// - `Leave`: The client has left the current room.
/// - `Error`: There was an error.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponsePacket {
    /// The client has joined a room.
    /// 
    /// The `size` field is an optional field that specifies the size of the room. If it is `None`,
    /// the size is unknown.
    Join {
        /// The size of the room. If it is `None`, the size is unknown.
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<usize>,
    },
    /// The client has created a new room.
    /// 
    /// The `id` field is the ID of the new room.
    Create {
        /// The ID of the new room.
        id: String,
    },
    /// The client has left the current room.
    /// 
    /// The `index` field is the index of the client in the room.
    Leave {
        /// The index of the client in the room.
        index: usize,
    },
    /// There was an error.
    /// 
    /// The `message` field is the error message.
    Error {
        /// The error message.
        message: String,
    },
}
