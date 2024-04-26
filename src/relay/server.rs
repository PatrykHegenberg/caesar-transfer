use axum::extract::ws::Message;
use futures_util::{future::join_all, stream::SplitSink, SinkExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, vec};
use tokio::{sync::Mutex, sync::RwLock};
use tracing::error;

use uuid::Uuid;

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
    senders: Vec<Sender>,
    size: usize,
}

impl Room {
    /// The default size of a room.
    ///
    /// This is the size that a room will have when it is created.
    const DEFAULT_ROOM_SIZE: usize = 2;

    /// Creates a new `Room` with the given size.
    ///
    /// The `size` parameter is the maximum number of clients that can join the
    /// room. If `size` is 0, then the room will not be able to hold any
    /// clients.
    ///
    /// The `senders` field of the returned `Room` is an empty vector.
    ///
    /// The `size` field of the returned `Room` is `size`.
    fn new(size: usize) -> Room {
        Room {
            // Initialize the list of senders to be empty.
            senders: Vec::new(),
            // Set the size of the room.
            size,
        }
    }
}

/// A struct that holds all of the rooms that the server knows about.
///
/// The rooms are stored in a `HashMap` with the room ID as the key and the
/// room as the value. This means that looking up a room by its ID is an O(1)
/// operation, which is very fast.
#[derive(Debug)]
pub struct Server {
    pub rooms: HashMap<String, Room>,
}

impl Server {
    /// Creates a new `Server` with an empty list of rooms.
    ///
    /// The `rooms` field of the returned `Server` is an empty `HashMap`.
    /// This means that the server will not have any rooms when it is first
    /// created.
    ///
    /// This function returns an `Arc<RwLock<Server>>` because the server
    /// needs to be shared between different parts of the program. The
    /// `Arc` makes it so that the server can be shared by multiple threads,
    /// and the `RwLock` makes it so that the server can be read from and
    /// written to from multiple threads at the same time.
    ///
    /// The `Arc` and `RwLock` are both parts of the `tokio` library, which
    /// provides asynchronous programming tools for Rust.
    ///
    /// The `Arc` and `RwLock` are used together to create a Mutex-like
    /// object that can be shared between threads. The main difference
    /// between a Mutex and an `Arc<RwLock<T>>` is that a Mutex can only be
    /// locked by one thread at a time, while an `Arc<RwLock<T>>` can be
    /// locked by multiple threads at the same time.
    ///
    /// This function is used to create a new `Server` and share it between
    /// different parts of the program. The `Server` is shared because it
    /// needs to be able to handle connections from multiple clients at the
    /// same time.
    pub fn new() -> Arc<RwLock<Server>> {
        // Create a new `Server` instance.
        Arc::new(RwLock::new(Server {
            // Initialize the list of rooms to be empty.
            rooms: HashMap::new(),
        }))
    }
}

/// This struct represents a single client connection to the server.
///
/// A `Client` instance holds a `Sender` and a `room_id`.
///
/// The `Sender` is a type alias for a `tokio::sync::mpsc::Sender<Message>`.
/// It is used to send messages to the client.
///
/// The `room_id` is an `Option<String>`. It is used to keep track of which
/// room the client is currently in. If the `room_id` is `None`, then the
/// client is not in any room. If the `room_id` is `Some(id)`, where `id` is a
/// `String`, then the client is in the room with the ID `id`.
///
/// The `room_id` is used to keep track of which room the client is in so
/// that the server knows which room to send messages to. When a client
/// joins a room, their `room_id` is set to the ID of the room that they
/// joined. When a client leaves a room, their `room_id` is set to `None`.
///
/// The `Client` struct is used to keep track of which room each client is
/// in. It is used by the `Server` to determine which room to send messages
/// to.
///
#[derive(Debug)]
pub struct Client {
    sender: Sender,
    room_id: Option<String>,
}

impl Client {
    /// Creates a new `Client` instance.
    ///
    /// The `sender` argument is a `Sender` for sending messages to the client.
    /// It is used by the `Server` to send messages to the client.
    ///
    /// The `room_id` field of the `Client` instance is set to `None` initially.
    /// This is because the client is not in any room when they first connect
    /// to the server.
    ///
    /// The `sender` field of the `Client` instance is used to send messages to
    /// the client. When the server wants to send a message to the client, it
    /// uses the `sender` to send the message.
    ///
    /// The `Client` instance is used by the `Server` to keep track of which
    /// room each client is in. It is used by the `Server` to determine which
    /// room to send messages to.
    pub fn new(sender: Sender) -> Client {
        Client {
            sender,
            room_id: None,
        }
    }

    /// Sends a message to a client.
    ///
    /// This function takes a `sender` argument, which is a `Mutex` guard
    /// for a WebSocket connection. The `sender` is used to send a message
    /// to the client.
    ///
    /// The `message` argument is the message that is sent to the client. It
    /// is a WebSocket message.
    ///
    /// This function locks the `sender` Mutex to ensure that only one thread
    /// can send a message at a time. This is because the SplitSink that the
    /// `sender` mutex guards is not thread-safe, and sending a message from
    /// multiple threads could result in the messages being sent out of order.
    ///
    /// If sending the message fails, this function logs an error message.
    async fn send(&self, sender: Sender, message: Message) {
        let mut sender = sender.lock().await;
        if let Err(error) = sender.send(message).await {
            error!("Failed to send message to the client: {}", error);
        }
    }

    /// Sends a packet to a client.
    ///
    /// This function takes a `sender` argument, which is a `Mutex` guard
    /// for a WebSocket connection. The `sender` is used to send a message
    /// to the client.
    ///
    /// The `packet` argument is the packet that is sent to the client. It
    /// is a struct that contains the data that is being sent.
    ///
    /// This function serializes the `packet` using serde_json and sends it
    /// to the client as a WebSocket Text message.
    ///
    /// This function locks the `sender` Mutex to ensure that only one thread
    /// can send a message at a time. This is because the SplitSink that the
    /// `sender` mutex guards is not thread-safe, and sending a message from
    /// multiple threads could result in the messages being sent out of order.
    async fn send_packet(&self, sender: Sender, packet: ResponsePacket) {
        let serialized_packet = serde_json::to_string(&packet).unwrap();

        self.send(sender, Message::Text(serialized_packet)).await;
    }

    /// Sends an error packet to a client.
    ///
    /// This function takes a `sender` argument, which is a `Mutex` guard
    /// for a WebSocket connection. The `sender` is used to send a message
    /// to the client.
    ///
    /// The `message` argument is the message that is sent to the client. It
    /// is a string that describes the error.
    ///
    /// This function creates an error packet with the `message` and sends it
    /// to the client using the `send_packet` function.
    ///
    /// This function locks the `sender` Mutex to ensure that only one thread
    /// can send a message at a time. This is because the SplitSink that the
    /// `sender` mutex guards is not thread-safe, and sending a message from
    /// multiple threads could result in the messages being sent out of order.
    async fn send_error_packet(&self, sender: Sender, message: String) {
        let error_packet = ResponsePacket::Error { message };

        self.send_packet(sender, error_packet).await
    }

    /// Handles a CreateRoom request from a client.
    ///
    /// This function is called when a client sends a CreateRoom request to
    /// the server. The server will create a new room with the specified
    /// size and return the room's identifier to the client.
    ///
    /// This function takes a `server` argument, which is a `RwLock`
    /// guard for the server's state. The `server` is used to check if the
    /// current client is already in a room, and to insert the new room into
    /// the server's state.
    ///
    /// If the current client is already in a room, this function returns
    /// without doing anything. This is to prevent a client from being in
    /// multiple rooms at the same time.
    ///
    /// If there is already a room with the same identifier as the one that
    /// is being created, this function sends an error packet to the client
    /// and returns.
    ///
    /// If there is no existing room with the same identifier, this function
    /// creates a new room with the specified size and inserts it into the
    /// server's state. It then sends a CreateRoom response packet to the
    /// client with the room's identifier.
    ///
    /// This function locks the `server` RwLock to ensure that only one
    /// thread can access the server's state at a time. This is because the
    /// server's state is not thread-safe, and accessing it from multiple
    /// threads could result in undefined behavior.
    async fn handle_create_room(&mut self, server: &RwLock<Server>) {
        let mut server = server.write().await;

        // If the current client is already in a room, do nothing.
        if server.rooms.iter().any(|(_, room)| {
            room.senders
                .iter()
                .any(|sender| Arc::ptr_eq(sender, &self.sender))
        }) {
            return;
        }

        // Generate a new room identifier.
        let size = Room::DEFAULT_ROOM_SIZE;
        let room_id = Uuid::new_v4().to_string();

        // If there is already a room with the same identifier, send an error
        // packet to the client and return.
        if server.rooms.contains_key(&room_id) {
            drop(server);

            return self
                .send_error_packet(
                    self.sender.clone(),
                    "A room with that identifier already exists.".to_string(),
                )
                .await;
        }

        // Create a new room with the specified size and insert it into the
        // server's state.
        let mut room = Room::new(size);
        room.senders.push(self.sender.clone());

        server.rooms.insert(room_id.clone(), room);

        // Set the client's room ID to the new room's identifier.
        self.room_id = Some(room_id.clone());

        drop(server);

        // Send a CreateRoom response packet to the client with the room's
        // identifier.
        self.send_packet(self.sender.clone(), ResponsePacket::Create { id: room_id })
            .await
    }

    /// This function is called when the client sends a JoinRoom packet.
    ///
    /// If the client is already in a room, then this function does nothing.
    ///
    /// If the client is not in a room, then the function checks if the room
    /// specified in the packet exists. If the room does not exist, an error
    /// packet is sent to the client with a message indicating that the room
    /// does not exist.
    ///
    /// If the room does exist, then the function checks if the room is full.
    /// If the room is full, an error packet is sent to the client with a
    /// message indicating that the room is full.
    ///
    /// If the room is not full, then the client is added to the room and the
    /// function sends a JoinRoom response packet to the client with the size
    /// of the room (excluding the client itself) and a `size` field set to
    /// `None`. The response packet is sent to all other clients in the room.
    async fn handle_join_room(&mut self, server: &RwLock<Server>, room_id: String) {
        let mut server = server.write().await;

        // If the client is already in a room, do nothing.
        if server.rooms.iter().any(|(_, room)| {
            room.senders
                .iter()
                .any(|sender| Arc::ptr_eq(sender, &self.sender))
        }) {
            return;
        }

        // Get a mutable reference to the room specified in the packet.
        // If the room does not exist, return an error to the client.
        let Some(room) = server.rooms.get_mut(&room_id) else {
            drop(server);

            return self
                .send_error_packet(self.sender.clone(), "The room does not exist.".to_string())
                .await;
        };

        // If the room is full, return an error to the client.
        if room.senders.len() >= room.size {
            drop(server);

            return self
                .send_error_packet(self.sender.clone(), "The room is full.".to_string())
                .await;
        }

        // Add the client to the room and set the client's room ID to the new
        // room's identifier.
        room.senders.push(self.sender.clone());
        self.room_id = Some(room_id);

        // Create a list of futures to send JoinRoom response packets to all
        // other clients in the room. The `size` field of the response packet is
        // set to `None` if the client sending the packet is the one joining the
        // room. Otherwise, the `size` field is set to the number of clients in
        // the room minus one (to exclude the client joining the room).
        let mut futures = vec![];
        for sender in &room.senders {
            if Arc::ptr_eq(sender, &self.sender) {
                futures.push(self.send_packet(
                    sender.clone(),
                    ResponsePacket::Join {
                        size: Some(room.senders.len() - 1),
                    },
                ));
            } else {
                futures.push(self.send_packet(sender.clone(), ResponsePacket::Join { size: None }));
            }
        }

        drop(server);
        join_all(futures).await;
    }

    /// Handles a request to leave a room.
    ///
    /// This function is called when a client sends a `LeaveRoom` request
    /// packet. The function obtains a write lock on the server's state and
    /// does the following:
    ///
    /// 1. Gets the room ID of the client who sent the request. If the client is
    ///    not in a room, the function returns early.
    /// 2. Tries to get a mutable reference to the room with the obtained room
    ///    ID. If the room does not exist, the function returns early.
    /// 3. Finds the index of the client's sender in the room's list of senders.
    ///    If the client is not in the room, the function returns early.
    /// 4. Removes the client's sender from the room's list of senders.
    /// 5. Sets the client's room ID to `None`.
    /// 6. Creates a list of futures to send `LeaveRoom` response packets to
    ///    all other clients in the room. The `index` field of the response
    ///    packet is set to the index of the client's sender in the room's list
    ///    of senders.
    /// 7. If the room is now empty, removes the room from the server's list
    ///    of rooms.
    /// 8. Drops the write lock on the server's state.
    /// 9. Waits for all futures to complete.
    async fn handle_leave_room(&mut self, server: &RwLock<Server>) {
        // Obtain a write lock on the server's state.
        let mut server = server.write().await;

        // Get the room ID of the client who sent the request.
        let Some(room_id) = self.room_id.clone() else {
            // If the client is not in a room, return early.
            return;
        };

        // Try to get a mutable reference to the room with the obtained room ID.
        let Some(room) = server.rooms.get_mut(&room_id) else {
            // If the room does not exist, return early.
            return;
        };

        // Find the index of the client's sender in the room's list of senders.
        let Some(index) = room
            .senders
            .iter()
            .position(|sender| Arc::ptr_eq(sender, &self.sender))
        else {
            // If the client is not in the room, return early.
            return;
        };

        // Remove the client's sender from the room's list of senders.
        room.senders.remove(index);

        // Set the client's room ID to `None`.
        self.room_id = None;

        // Create a list of futures to send `LeaveRoom` response packets to
        // all other clients in the room. The `index` field of the response
        // packet is set to the index of the client's sender in the room's list
        // of senders.
        let mut futures = vec![];
        for sender in &room.senders {
            futures.push(self.send_packet(sender.clone(), ResponsePacket::Leave { index }));
        }

        // If the room is now empty, removes the room from the server's list
        // of rooms.
        if room.senders.is_empty() {
            server.rooms.remove(&room_id);
        }

        // Drop the write lock on the server's state.
        drop(server);

        // Wait for all futures to complete.
        join_all(futures).await;
    }

    /// This function handles an incoming message from a client.
    ///
    /// The message can be one of four types: `Text`, `Binary`, `Ping`, or `Close`.
    ///
    /// If the message is `Text`, the function parses the message as a `RequestPacket` and
    /// calls the appropriate function to handle the request. If the message cannot be
    /// parsed as a `RequestPacket`, the function does nothing and returns early.
    ///
    /// If the message is `Binary`, the function first acquires a read lock on the server's
    /// state. If the client is not currently in a room, the function drops the read lock and
    /// returns early. If the client is not in a room, or if the room does not exist, the
    /// function drops the read lock and returns early.
    ///
    /// The function then finds the index of the client's sender in the room's list of
    /// senders. If the client's sender is not in the room's list of senders, the function
    /// drops the read lock and returns early.
    ///
    /// The function then gets the binary data from the message and sets the first byte to
    /// the index of the client's sender in the room's list of senders. If there is no
    /// binary data in the message, the function drops the read lock and returns early.
    ///
    /// The function then determines where to send the message. If the first byte of the
    /// message is less than the number of clients in the room, the function sends the message
    /// to the client at that index in the room's list of senders. If the first byte of the
    /// message is equal to the number of clients in the room plus one, the function sends the
    /// message to all clients in the room, excluding the client that sent the message.
    ///
    /// If the first byte of the message is any other value, the function drops the read
    /// lock and returns early.
    ///
    /// Finally, the function drops the read lock and waits for all futures to complete.
    ///
    /// If the message is `Ping`, the function prints a message to stdout.
    ///
    /// If the message is `Pong`, the function prints a message to stdout.
    ///
    /// If the message is `Close`, the function prints a message to stdout and calls the
    /// `handle_close` function.
    pub async fn handle_message(&mut self, server: &RwLock<Server>, message: Message) {
        match message {
            Message::Text(text) => {
                let packet = match serde_json::from_str(&text) {
                    Ok(packet) => packet,
                    Err(_) => return,
                };
                match packet {
                    RequestPacket::Create => self.handle_create_room(server).await,
                    RequestPacket::Join { id } => self.handle_join_room(server, id).await,
                    RequestPacket::Leave => self.handle_leave_room(server).await,
                }
            }
            Message::Binary(_) => {
                // Acquire a read lock on the server's state.
                let server = server.read().await;

                // If the client is not currently in a room, return early.
                let Some(room_id) = &self.room_id else {
                    drop(server);
                    return;
                };

                // If the room does not exist, return early.
                let Some(room) = server.rooms.get(room_id) else {
                    drop(server);
                    return;
                };

                // Find the index of the client's sender in the room's list of senders.
                let Some(index) = room
                    .senders
                    .iter()
                    .position(|sender| Arc::ptr_eq(sender, &self.sender))
                else {
                    drop(server);
                    return;
                };

                // Get the binary data from the message and set the first byte to
                // the index of the client's sender in the room's list of senders.
                let mut data = message.into_data();
                if data.is_empty() {
                    drop(server);
                    return;
                }

                let source = u8::try_from(index).unwrap();

                // Determine where to send the message.
                let destination = usize::from(data[0]);
                data[0] = source;

                // Send the message to the client at the destination index in the
                // room's list of senders.
                if destination < room.senders.len() {
                    let sender = room.senders[destination].clone();

                    drop(server);
                    return self.send(sender, Message::Binary(data)).await;
                }

                // Send the message to all clients in the room, excluding the
                // client that sent the message.
                if destination == usize::from(u8::MAX) {
                    let mut futures = vec![];
                    for sender in &room.senders {
                        if Arc::ptr_eq(sender, &self.sender) {
                            continue;
                        }

                        futures.push(self.send(sender.clone(), Message::Binary(data.clone())));
                    }

                    drop(server);
                    join_all(futures).await;
                }
            }
            Message::Ping(_) => {
                println!("Got Message Type Ping");
            }
            Message::Pong(_) => {
                println!("Got Message Type Pong");
            }
            Message::Close(_) => {
                println!("Got Message Type Close");
                self.handle_close(server).await;
            }
        }
    }

    pub async fn handle_close(&mut self, server: &RwLock<Server>) {
        self.handle_leave_room(server).await
    }
}
