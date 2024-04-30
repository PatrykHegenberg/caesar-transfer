use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transfer {
    pub name: String,
    pub ip: String,
    pub room_id: String,
}

impl Transfer {
    pub fn new(name: String, ip: String, room_id: String) -> Self {
        Self { name, ip, room_id }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let transfer = Transfer {
            name: "Test".to_string(),
            ip: "127.0.0.1".to_string(),
            room_id: "This_is_a_test_room_id".to_string(),
        };
        assert_eq!(
            Transfer::new(
                "Test".to_string(),
                "127.0.0.1".to_string(),
                "This_is_a_test_room_id".to_string(),
            ),
            transfer
        )
    }
}
