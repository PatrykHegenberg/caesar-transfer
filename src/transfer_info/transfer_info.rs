use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferInfoRequest {
    pub ip: String,
    pub name: String,
    pub body: TransferInfoBody,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferInfoBody {
    pub keyword: String,
    pub files: String,
}

impl TransferInfoRequest {
    pub fn new() -> Self {
        Self {
            ip: "".to_string(),
            name: "".to_string(),
            body: TransferInfoBody {
                keyword: "".to_string(),
                files: "".to_string(),
            },
        }
    }
}
