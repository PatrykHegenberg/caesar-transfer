use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferRequest {
    pub name: String,
    pub ip: String,
    pub local_room_id: String,
    pub relay_room_id: String,
}
impl TransferRequest {
    pub fn new(name: String, ip: String, local_room_id: String, relay_room_id: String) -> Self {
        Self {
            name,
            ip,
            local_room_id,
            relay_room_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferResponse {
    pub name: String,
    pub ip: String,
    pub local_room_id: String,
    pub relay_room_id: String,
}

impl TransferResponse {
    pub fn new(name: String, ip: String, local_room_id: String, relay_room_id: String) -> Self {
        Self {
            name,
            ip,
            local_room_id,
            relay_room_id,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let transfer = TransferResponse {
            name: "Test".to_string(),
            ip: "127.0.0.1".to_string(),
            local_room_id: "This_is_a_test_room_id".to_string(),
            relay_room_id: "This_is_a_test_room_id".to_string(),
        };
        assert_eq!(
            TransferResponse::new(
                "Test".to_string(),
                "127.0.0.1".to_string(),
                "This_is_a_test_room_id".to_string(),
                "This_is_a_test_room_id".to_string(),
            ),
            transfer
        )
    }
}
