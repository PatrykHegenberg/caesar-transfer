use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TransferNotCreatedError {
    pub message: String,
}

impl TransferNotCreatedError {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for TransferNotCreatedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TransferNotCreatedError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug)]
pub struct TransferNotFoundError {
    message: String,
}

impl TransferNotFoundError {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for TransferNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TransferNotFoundError {
    fn description(&self) -> &str {
        &self.message
    }
}
