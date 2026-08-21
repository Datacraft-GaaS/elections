use serde::{Deserialize, Serialize};

/// Raw biometric data block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiometricDataBlock {
    pub data: Vec<u8>,
}

impl BiometricDataBlock {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
