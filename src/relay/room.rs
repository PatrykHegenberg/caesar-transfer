use axum::extract::ws::Message;
use futures_util::stream::SplitSink;
use std::sync::Arc;
use tokio::sync::Mutex;

// A type alias for a sender to a WebSocket connection.
//
// The sender is a mutex-guarded, split sink of a WebSocket stream and Message
// values. It is used to send messages to a client.
//
// The Mutex is used to ensure that only one thread can send a message at a
// time. This is because the SplitSink is not thread-safe, and sending a
// message from multiple threads could result in the messages being sent
// out of order.
//
// The SplitSink is used to send messages to a client. It is the part of the
// WebSocket stream that handles the sending of messages.
//
// The WebSocket stream is the underlying connection to the client. It is used
// to send and receive messages.
//
// The Message value is the type of data that is sent over the WebSocket
// connection. It is a struct that contains the data that is being sent.
//
// The type alias is used so that the type is not mentioned every time it is
// used. This makes the code easier to read and understand.
type Sender = Arc<Mutex<SplitSink<axum::extract::ws::WebSocket, Message>>>;

/// A `Room` is a collection of clients that are connected to each other.
///
/// Each room has a set of clients, represented by a `Vec` of `Sender`
/// instances. The `Sender` instances are used to send messages to the
/// clients in the room.
///
/// The `senders` field is the list of senders that are connected to each
/// other. Each sender is a mutex-guarded, split sink of a WebSocket
/// stream and Message values. This is explained in more detail in the
/// documentation for the `Sender` type alias in the `packets` module.
///
/// The `size` field is the maximum number of clients that a room can have.
/// When a room reaches its maximum size, no more clients can join the room.
/// This is used to prevent rooms from getting too full and causing the
/// server to run out of memory.
#[derive(Debug)]
pub struct Room {
    pub senders: Vec<Sender>,
    pub size: usize,
}

impl Room {
    /// The default size of a room.
    ///
    /// This is the size that a room will have when it is created.
    pub const DEFAULT_ROOM_SIZE: usize = 2;

    /// Creates a new `Room` with the given size.
    ///
    /// The `size` parameter is the maximum number of clients that can join the
    /// room. If `size` is 0, then the room will not be able to hold any
    /// clients.
    ///
    /// The `senders` field of the returned `Room` is an empty vector.
    ///
    /// The `size` field of the returned `Room` is `size`.
    pub fn new(size: usize) -> Room {
        Room {
            // Initialize the list of senders to be empty.
            senders: Vec::new(),
            // Set the size of the room.
            size,
        }
    }
}
