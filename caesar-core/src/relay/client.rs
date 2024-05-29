use axum::extract::ws::Message;
use futures_util::{future::join_all, stream::SplitSink, SinkExt};
use std::{sync::Arc, vec};
use tokio::{sync::Mutex, sync::RwLock};
use tracing::{debug, error};

use crate::relay::appstate::AppState;
use crate::relay::room::Room;
use crate::relay::RequestPacket;
use crate::relay::ResponsePacket;
use uuid::Uuid;

/// Type alias for a synchronized WebSocket sender.
/// 
/// This is used to send messages to a WebSocket connection.
type Sender = Arc<Mutex<SplitSink<axum::extract::ws::WebSocket, Message>>>;

/// Struct representing a WebSocket client.
/// 
/// This struct contains a WebSocket sender and an optional room ID.
/// The sender is used to send messages to the WebSocket connection,
/// while the room ID is used to identify the client's room.
#[derive(Debug)]
pub struct Client {
    /// The WebSocket sender for sending messages.
    sender: Sender,
    /// The optional room ID of the client.
    /// 
    /// This is used to identify the client's room.
    room_id: Option<String>,
}

impl Client {
    /// Creates a new WebSocket client.
    ///
    /// # Arguments
    ///
    /// * `sender` - A synchronized WebSocket sender.
    ///
    /// # Returns
    ///
    /// A new WebSocket client instance.
    pub fn new(sender: Sender) -> Client {
        Client {
            sender, // The WebSocket sender for sending messages.
            room_id: None, // The optional room ID of the client. This is used to identify the client's room.
        }
    }

    /// Sends a message to the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `sender` - A synchronized WebSocket sender.
    /// * `message` - The message to send.
    ///
    /// # Errors
    ///
    /// If the message fails to be sent.
    async fn send(&self, sender: Sender, message: Message) {
        let mut sender = sender.lock().await; // Acquires a lock on the sender.
        if let Err(error) = sender.send(message).await { // Sends the message.
            error!("Failed to send message to the client: {}", error); // Logs the error if the message fails to be sent.
        }
    }

    /// Sends a serialized packet to the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `sender` - A synchronized WebSocket sender.
    /// * `packet` - The packet to send.
    ///
    /// # Errors
    ///
    /// If the serialized packet fails to be sent.
    async fn send_packet(&self, sender: Sender, packet: ResponsePacket) {
        // Serialize the packet to a string.
        let serialized_packet = serde_json::to_string(&packet).unwrap();

        // Send the serialized packet as a text message.
        self.send(sender, Message::Text(serialized_packet)).await;
    }

    /// Sends an error message to the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `sender` - A synchronized WebSocket sender.
    /// * `message` - The error message to send.
    ///
    /// # Errors
    ///
    /// If the error message fails to be sent.
    async fn send_error_packet(&self, sender: Sender, message: String) {
        // Create an error packet with the given message.
        let error_packet = ResponsePacket::Error { message };

        // Send the error packet.
        self.send_packet(sender, error_packet).await;
    }

    /// Handles the "create_room" request from a client.
    ///
    /// # Arguments
    ///
    /// * `server` - A lock guard of the `AppState`.
    /// * `id` - An optional string representing the room identifier.
    ///
    /// # Errors
    ///
    /// If the room already exists or if the room creation fails.
    async fn handle_create_room(&mut self, server: &RwLock<AppState>, id: Option<String>) {
        // Acquire a write lock on the server state.
        let mut server = server.write().await;

        // Check if the client is already in a room.
        if server.rooms.iter().any(|(_, room)| {
            room.senders
                .iter()
                .any(|sender| Arc::ptr_eq(sender, &self.sender))
        }) {
            return;
        }

        // Set the room size and generate a room identifier if none is provided.
        let size = Room::DEFAULT_ROOM_SIZE;
        let room_id = match id {
            Some(id) => id,
            None => Uuid::new_v4().to_string(),
        };

        // Check if the room identifier already exists.
        if server.rooms.contains_key(&room_id) {
            drop(server); // Release the lock before returning.

            return self
                .send_error_packet(
                    self.sender.clone(),
                    "A room with that identifier already exists.".to_string(),
                )
                .await;
        }

        // Create a new room and add the client to it.
        let mut room = Room::new(size);
        room.senders.push(self.sender.clone());

        // Insert the room into the server state.
        server.rooms.insert(room_id.clone(), room);

        self.room_id = Some(room_id.clone()); // Store the room identifier.

        drop(server); // Release the lock before returning.

        debug!("Room created");
        // Send the response packet to the client.
        self.send_packet(self.sender.clone(), ResponsePacket::Create { id: room_id })
            .await
    }

    /// Handles the "join_room" request from a client.
    ///
    /// # Arguments
    ///
    /// * `server` - A lock guard of the `AppState`.
    /// * `room_id` - A string representing the room identifier.
    ///
    /// # Errors
    ///
    /// If the room does not exist or if the room is full.
    async fn handle_join_room(&mut self, server: &RwLock<AppState>, room_id: String) {
        let mut server = server.write().await;

        // Check if the client is already in a room.
        if server.rooms.iter().any(|(_, room)| {
            room.senders
                .iter()
                .any(|sender| Arc::ptr_eq(sender, &self.sender))
        }) {
            return;
        }

        let Some(room) = server.rooms.get_mut(&room_id) else {
            drop(server);

            // Send an error packet to the client.
            return self
                .send_error_packet(self.sender.clone(), "The room does not exist.".to_string())
                .await;
        };

        // Check if the room is full.
        if room.senders.len() >= room.size {
            drop(server);

            // Send an error packet to the client.
            return self
                .send_error_packet(self.sender.clone(), "The room is full.".to_string())
                .await;
        }

        // Add the client to the room.
        room.senders.push(self.sender.clone());
        self.room_id = Some(room_id);

        let mut futures = vec![];
        for sender in &room.senders {
            // Send a join packet to the client with its position in the room.
            if Arc::ptr_eq(sender, &self.sender) {
                futures.push(self.send_packet(
                    sender.clone(),
                    ResponsePacket::Join {
                        size: Some(room.senders.len() - 1),
                    },
                ));
            } else {
                // Send a join packet to the client without its position in the room.
                futures.push(self.send_packet(sender.clone(), ResponsePacket::Join { size: None }));
            }
        }

        drop(server);
        join_all(futures).await;
    }

    /// Handle the leave room request from the client.
    ///
    /// This function removes the client from the current room and notifies the other
    /// clients in the room about the client's departure.
    ///
    /// # Arguments
    ///
    /// * `server` - A read-write lock guard for the server state.
    ///
    /// # Returns
    ///
    /// This function does not return anything.
    #[allow(clippy::needless_pass_by_value)]
    async fn handle_leave_room(&mut self, server: &RwLock<AppState>) {
        // Acquire a write lock on the server state.
        let mut server = server.write().await;

        // Get the room ID of the current room.
        let Some(room_id) = self.room_id.clone() else {
            return;
        };

        // Get the mutable reference to the room.
        let Some(room) = server.rooms.get_mut(&room_id) else {
            return;
        };

        // Get the index of the client in the room.
        let Some(index) = room
            .senders
            .iter()
            .position(|sender| Arc::ptr_eq(sender, &self.sender))
        else {
            return;
        };

        // Remove the client from the room.
        room.senders.remove(index);

        self.room_id = None;

        let mut futures = vec![];
        for sender in &room.senders {
            // Send a leave packet to the other clients in the room.
            futures.push(self.send_packet(sender.clone(), ResponsePacket::Leave { index }));
        }

        // If the room is empty, remove it from the server state.
        if room.senders.is_empty() {
            server.rooms.remove(&room_id);
        }

        drop(server);

        // Wait for all the futures to complete.
        join_all(futures).await;
    }

    /// Handles incoming messages from the client.
    ///
    /// This function interprets the incoming message and performs the corresponding action.
    ///
    /// # Arguments
    ///
    /// * `server` - A RwLock guard containing the state of the server.
    /// * `message` - The incoming message from the client.
    pub async fn handle_message(&mut self, server: &RwLock<AppState>, message: Message) {
        // Match on the type of the message.
        match message {
            // If the message is text, parse it as a RequestPacket.
            Message::Text(text) => {
                let packet = match serde_json::from_str(&text) {
                    Ok(packet) => packet,
                    Err(_) => return, // Return if the parsing fails.
                };
                // Match on the RequestPacket type and perform the corresponding action.
                match packet {
                    RequestPacket::Create { id } => self.handle_create_room(server, id).await,
                    RequestPacket::Join { id } => self.handle_join_room(server, id).await,
                    RequestPacket::Leave => self.handle_leave_room(server).await,
                }
            }
            // If the message is binary, handle it accordingly.
            Message::Binary(_) => {
                // Acquire a read lock on the server state.
                let server = server.read().await;

                // Get the room ID of the current room.
                let Some(room_id) = &self.room_id else {
                    drop(server);
                    return; // Return if the client is not in a room.
                };

                // Get the room corresponding to the room ID.
                let Some(room) = server.rooms.get(room_id) else {
                    drop(server);
                    return; // Return if the room does not exist.
                };

                // Get the index of the client in the room.
                let Some(index) = room
                    .senders
                    .iter()
                    .position(|sender| Arc::ptr_eq(sender, &self.sender))
                else {
                    drop(server);
                    return; // Return if the client is not in the room.
                };

                // Get the binary data from the message.
                let mut data = message.into_data();
                if data.is_empty() {
                    drop(server);
                    return; // Return if the data is empty.
                }

                // Convert the index to a u8 and assign it as the source.
                let source = u8::try_from(index).unwrap();

                // Get the destination from the first byte of the data.
                let destination = usize::from(data[0]);
                data[0] = source; // Assign the source to the first byte of the data.

                // If the destination is within the range of the room senders, send the data to that sender.
                if destination < room.senders.len() {
                    let sender = room.senders[destination].clone();

                    drop(server);
                    return self.send(sender, Message::Binary(data)).await;
                }

                // If the destination is u8::MAX, send the data to all the room senders except the current one.
                if destination == usize::from(u8::MAX) {
                    let mut futures = vec![];
                    for sender in &room.senders {
                        if Arc::ptr_eq(sender, &self.sender) {
                            continue; // Skip the current client.
                        }

                        futures.push(self.send(sender.clone(), Message::Binary(data.clone())));
                    }

                    drop(server);
                    join_all(futures).await;
                }
            }
            // If the message is Ping, print a message.
            Message::Ping(_) => {
                println!("Got Message Type Ping");
            }
            // If the message is Pong, print a message.
            Message::Pong(_) => {
                println!("Got Message Type Pong");
            }
            // If the message is Close, print a message and handle the close.
            Message::Close(_) => {
                println!("Got Message Type Close");
                self.handle_close(server).await;
            }
        }
    }

    pub async fn handle_close(&mut self, server: &RwLock<AppState>) {
        self.handle_leave_room(server).await
    }
}
// TODO: Add tests
#[cfg(test)]
mod tests {
    // use super::*;
}
