use crate::IcaoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintData {
    /// Finger position code per ANSI/NIST-ITL: 1=right thumb, 2=right index, etc.
    pub position: u8,
    pub image_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleFingerprintData {
    pub records: Vec<FingerprintData>,
}

impl MultipleFingerprintData {
    /// Parse DG3 fingerprint biometric data group.
    ///
    /// Minimal parsing: first byte is finger position, remainder is image data.
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.is_empty() {
            return Err(IcaoError::InvalidLength);
        }
        let position = data[0];
        let image_data = data[1..].to_vec();
        Ok(Self {
            records: vec![FingerprintData {
                position,
                image_data,
            }],
        })
    }
}
