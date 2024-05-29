use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::relay::room::Room;
use crate::relay::transfer::TransferResponse;

/// State of the application.
///
/// This structure holds the state of the application, which includes the rooms
/// and the transfers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Map of rooms, where the key is the room's ID and the value is the room
    /// itself.
    pub rooms: HashMap<String, Room>,
    /// Vector of transfers.
    pub transfers: Vec<TransferResponse>,
}

impl AppState {
    /// Creates a new instance of the `AppState` struct.
    ///
    /// This function initializes the state of the application with an empty map
    /// of rooms and an empty vector of transfers.
    ///
    /// # Returns
    ///
    /// An `Arc<RwLock<AppState>>` that can be used to share the state across multiple
    /// tasks.
    pub fn new() -> Arc<RwLock<AppState>> {
        // Create a new instance of `AppState` with empty rooms and transfers.
        let app_state = AppState {
            rooms: HashMap::new(),
            transfers: Vec::new(),
        };

        // Wrap the `app_state` in a `RwLock` to make it thread-safe.
        Arc::new(RwLock::new(app_state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_new() {
        let app_state = AppState::new();

        assert!(Arc::ptr_eq(&app_state, &app_state.clone()));
    }
}
