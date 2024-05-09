use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::relay::room::Room;
use crate::relay::transfer::TransferResponse;

#[derive(Debug, Clone)]
pub struct AppState {
    pub rooms: HashMap<String, Room>,
    pub transfers: Vec<TransferResponse>,
}

impl AppState {
    pub fn new() -> Arc<RwLock<AppState>> {
        Arc::new(RwLock::new(AppState {
            rooms: HashMap::new(),
            transfers: Vec::new(),
        }))
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
