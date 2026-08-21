use serde::{Deserialize, Serialize};

/// Optional security block appended after the BDB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityBlock {
    pub algorithm_id: u16,
    pub data: Vec<u8>,
}

impl SecurityBlock {
    pub fn new(algorithm_id: u16, data: Vec<u8>) -> Self {
        Self { algorithm_id, data }
    }
}
