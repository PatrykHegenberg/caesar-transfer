use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use std::sync::Arc;
use tokio::sync::Mutex;

type Sender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

#[derive(Debug, Clone)]
pub struct Room {
    pub senders: Vec<Sender>,
    pub size: usize,
}

impl Room {
    pub const DEFAULT_ROOM_SIZE: usize = 2;

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
