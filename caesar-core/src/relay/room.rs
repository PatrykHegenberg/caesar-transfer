use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use std::sync::Arc;
use tokio::sync::Mutex;

// `Sender` is a type alias for a synchronized WebSocket sender.
//
// This is used to send messages to a WebSocket connection.
type Sender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

/// Struct representing a room of WebSocket clients.
///
/// A `Room` contains a list of WebSocket senders and a room size.
/// The senders are used to send messages to the WebSocket connections,
/// while the room size represents the maximum number of clients allowed in the room.
#[derive(Debug, Clone)]
pub struct Room {
    /// The list of WebSocket senders.
    ///
    /// Each sender is used to send messages to a WebSocket connection.
    pub senders: Vec<Sender>,
    /// The size of the room.
    ///
    /// This represents the maximum number of clients allowed in the room.
    pub size: usize,
}

impl Room {
    /// The default room size.
    ///
    /// This is used as a fallback value when creating a new room.
    pub const DEFAULT_ROOM_SIZE: usize = 2;

    /// Create a new room with the specified size.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the room.
    ///
    /// # Returns
    ///
    /// A new `Room` instance.
    pub fn new(size: usize) -> Room {
        Room {
            senders: Vec::new(),
            size,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_new() {
        let room = Room::new(5);

        assert_eq!(room.size, 5);

        assert!(room.senders.is_empty());
    }
}
