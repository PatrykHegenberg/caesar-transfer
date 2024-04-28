pub mod appstate;
pub mod client;
pub mod room;
pub mod server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
// This enum is used to represent the different types of requests that a client
// can send to the server.
//
// The requests that a client can send are:
//
// * Join: A request to join a room. The request contains the ID of the room
// that the client wants to join.
// * Create: A request to create a new room.
// * Leave: A request to leave the current room.
pub enum RequestPacket {
    Join {
        // The ID of the room that the client wants to join.
        id: String,
    },
    Create,
    Leave,
}

/// This enum is used to represent the different types of responses that the
/// server can send to the client.
///
/// The responses that the server can send are:
///
/// * Join: A response to a `Join` request from the client. If the client
/// successfully joined a room, the `size` field will be `Some` and contain
/// the size of the room. If the client could not join a room, the `size` field
/// will be `None`.
/// * Create: A response to a `Create` request from the client. If the server
/// successfully created a room, the `id` field will contain the ID of the
/// room. If the server could not create a room, the `id` field will be empty.
/// * Leave: A response to a `Leave` request from the client. If the client
/// successfully left a room, the `index` field will contain the index of the
/// client that left the room. If the client could not leave a room, the
/// `index` field will be 0.
/// * Error: A response to indicate that an error occurred. The `message`
/// field will contain a description of the error.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponsePacket {
    Join {
        /// The size of the room that the client joined. If the client could
        /// not join a room, this field will be `None`.
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<usize>,
    },
    Create {
        /// The ID of the room that the server created. If the server could
        /// not create a room, this field will be empty.
        id: String,
    },
    Leave {
        /// The index of the client that left the room. If the client could not
        /// leave a room, this field will be 0.
        index: usize,
    },
    Error {
        /// A description of the error that occurred.
        message: String,
    },
}
