use crate::IcaoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAuthPublicKey {
    /// Algorithm OID string (e.g. "1.2.840.113549.1.1.1" for RSA).
    pub algorithm: String,
    /// Raw DER-encoded public key bytes.
    pub key_data: Vec<u8>,
}

impl ActiveAuthPublicKey {
    /// Parse DG15 Active Authentication public key.
    ///
    /// Minimal parsing: first byte is algorithm identifier length, followed by
    /// that many bytes of algorithm OID text, then remaining bytes are key data.
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.is_empty() {
            return Err(IcaoError::InvalidLength);
        }
        let oid_len = data[0] as usize;
        if data.len() < 1 + oid_len {
            return Err(IcaoError::InvalidLength);
        }
        let algorithm = std::str::from_utf8(&data[1..1 + oid_len])
            .map_err(|_| IcaoError::UnsupportedFormat("DG15 algorithm OID not UTF-8".into()))?
            .to_string();
        let key_data = data[1 + oid_len..].to_vec();
        Ok(Self {
            algorithm,
            key_data,
        })
    }
}
