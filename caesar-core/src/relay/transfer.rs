use serde::{Deserialize, Serialize};

/// Request to transfer a connection from one relay to another
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferRequest {
    /// The name of the client
    pub name: String,
    /// The IP address of the client
    pub ip: String,
    /// The local room ID of the client
    pub local_room_id: String,
    /// The relay room ID of the client
    pub relay_room_id: String,
}

impl TransferRequest {
    /// Creates a new transfer request
    ///
    /// # Args
    ///
    /// * `name` - The name of the client
    /// * `ip` - The IP address of the client
    /// * `local_room_id` - The local room ID of the client
    /// * `relay_room_id` - The relay room ID of the client
    ///
    /// # Returns
    ///
    /// A new `TransferRequest` instance
    pub fn new(name: String, ip: String, local_room_id: String, relay_room_id: String) -> Self {
        Self {
            name,
            ip,
            local_room_id,
            relay_room_id,
        }
    }
}

/// Response containing the details of the transferred connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferResponse {
    /// The name of the client
    pub name: String,
    /// The IP address of the client
    pub ip: String,
    /// The local room ID of the client
    pub local_room_id: String,
    /// The relay room ID of the client
    pub relay_room_id: String,
}

impl TransferResponse {
    /// Creates a new transfer response
    ///
    /// # Args
    ///
    /// * `name` - The name of the client
    /// * `ip` - The IP address of the client
    /// * `local_room_id` - The local room ID of the client
    /// * `relay_room_id` - The relay room ID of the client
    ///
    /// # Returns
    ///
    /// A new `TransferResponse` instance
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
