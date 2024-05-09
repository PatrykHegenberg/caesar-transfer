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

type Sender = Arc<Mutex<SplitSink<axum::extract::ws::WebSocket, Message>>>;
#[derive(Debug)]
pub struct Client {
    sender: Sender,
    room_id: Option<String>,
}

impl Client {
    pub fn new(sender: Sender) -> Client {
        Client {
            sender,
            room_id: None,
        }
    }

    async fn send(&self, sender: Sender, message: Message) {
        let mut sender = sender.lock().await;
        if let Err(error) = sender.send(message).await {
            error!("Failed to send message to the client: {}", error);
        }
    }

    async fn send_packet(&self, sender: Sender, packet: ResponsePacket) {
        let serialized_packet = serde_json::to_string(&packet).unwrap();

        self.send(sender, Message::Text(serialized_packet)).await;
    }

    async fn send_error_packet(&self, sender: Sender, message: String) {
        let error_packet = ResponsePacket::Error { message };

        self.send_packet(sender, error_packet).await
    }

    async fn handle_create_room(&mut self, server: &RwLock<AppState>, id: Option<String>) {
        let mut server = server.write().await;

        if server.rooms.iter().any(|(_, room)| {
            room.senders
                .iter()
                .any(|sender| Arc::ptr_eq(sender, &self.sender))
        }) {
            return;
        }

        let size = Room::DEFAULT_ROOM_SIZE;
        let room_id = match id {
            Some(id) => id,
            None => Uuid::new_v4().to_string(),
        };

        if server.rooms.contains_key(&room_id) {
            drop(server);

            return self
                .send_error_packet(
                    self.sender.clone(),
                    "A room with that identifier already exists.".to_string(),
                )
                .await;
        }

        let mut room = Room::new(size);
        room.senders.push(self.sender.clone());

        server.rooms.insert(room_id.clone(), room);

        self.room_id = Some(room_id.clone());

        drop(server);

        debug!("Room created");
        self.send_packet(self.sender.clone(), ResponsePacket::Create { id: room_id })
            .await
    }

    async fn handle_join_room(&mut self, server: &RwLock<AppState>, room_id: String) {
        let mut server = server.write().await;

        if server.rooms.iter().any(|(_, room)| {
            room.senders
                .iter()
                .any(|sender| Arc::ptr_eq(sender, &self.sender))
        }) {
            return;
        }

        let Some(room) = server.rooms.get_mut(&room_id) else {
            drop(server);

            return self
                .send_error_packet(self.sender.clone(), "The room does not exist.".to_string())
                .await;
        };

        if room.senders.len() >= room.size {
            drop(server);

            return self
                .send_error_packet(self.sender.clone(), "The room is full.".to_string())
                .await;
        }

        room.senders.push(self.sender.clone());
        self.room_id = Some(room_id);

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

    async fn handle_leave_room(&mut self, server: &RwLock<AppState>) {
        let mut server = server.write().await;

        let Some(room_id) = self.room_id.clone() else {
            return;
        };

        let Some(room) = server.rooms.get_mut(&room_id) else {
            return;
        };

        let Some(index) = room
            .senders
            .iter()
            .position(|sender| Arc::ptr_eq(sender, &self.sender))
        else {
            return;
        };

        room.senders.remove(index);

        self.room_id = None;

        let mut futures = vec![];
        for sender in &room.senders {
            futures.push(self.send_packet(sender.clone(), ResponsePacket::Leave { index }));
        }

        if room.senders.is_empty() {
            server.rooms.remove(&room_id);
        }

        drop(server);

        join_all(futures).await;
    }

    pub async fn handle_message(&mut self, server: &RwLock<AppState>, message: Message) {
        match message {
            Message::Text(text) => {
                let packet = match serde_json::from_str(&text) {
                    Ok(packet) => packet,
                    Err(_) => return,
                };
                match packet {
                    RequestPacket::Create { id } => self.handle_create_room(server, id).await,
                    RequestPacket::Join { id } => self.handle_join_room(server, id).await,
                    RequestPacket::Leave => self.handle_leave_room(server).await,
                }
            }
            Message::Binary(_) => {
                let server = server.read().await;

                let Some(room_id) = &self.room_id else {
                    drop(server);
                    return;
                };

                let Some(room) = server.rooms.get(room_id) else {
                    drop(server);
                    return;
                };

                let Some(index) = room
                    .senders
                    .iter()
                    .position(|sender| Arc::ptr_eq(sender, &self.sender))
                else {
                    drop(server);
                    return;
                };

                let mut data = message.into_data();
                if data.is_empty() {
                    drop(server);
                    return;
                }

                let source = u8::try_from(index).unwrap();

                let destination = usize::from(data[0]);
                data[0] = source;

                if destination < room.senders.len() {
                    let sender = room.senders[destination].clone();

                    drop(server);
                    return self.send(sender, Message::Binary(data)).await;
                }

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

    pub async fn handle_close(&mut self, server: &RwLock<AppState>) {
        self.handle_leave_room(server).await
    }
}
// TODO: Add tests
#[cfg(test)]
mod tests {
    // use super::*;
}
