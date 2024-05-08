pub mod appstate;
pub mod client;
pub mod room;
pub mod server;
pub mod transfer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RequestPacket {
    Join {
        // The ID of the room that the client wants to join.
        id: String,
    },
    Create {
        id: Option<String>,
    },
    Leave,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResponsePacket {
    Join {
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<usize>,
    },
    Create {
        id: String,
    },
    Leave {
        index: usize,
    },
    Error {
        message: String,
    },
}
