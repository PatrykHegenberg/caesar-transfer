use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::relay::room::Room;

/// A struct that holds all of the rooms that the server knows about.
///
/// The rooms are stored in a `HashMap` with the room ID as the key and the
/// room as the value. This means that looking up a room by its ID is an O(1)
/// operation, which is very fast.
#[derive(Debug)]
pub struct AppState {
    pub rooms: HashMap<String, Room>,
}

impl AppState {
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
    pub fn new() -> Arc<RwLock<AppState>> {
        // Create a new `Server` instance.
        Arc::new(RwLock::new(AppState {
            // Initialize the list of rooms to be empty.
            rooms: HashMap::new(),
        }))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc};

    #[test]
    fn test_new() {
        let app_state = AppState::new();

        assert!(Arc::ptr_eq(&app_state, &app_state.clone()));
    }
}