use crate::IcaoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOptions {
    /// OID strings for supported protocols (e.g. PACE, Chip Authentication, Terminal Auth).
    pub supported_protocols: Vec<String>,
}

impl SecurityOptions {
    /// Parse DG14 security options. Treats each null-delimited or newline-delimited
    /// segment as a protocol identifier; empty input yields an empty list.
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.is_empty() {
            return Ok(Self {
                supported_protocols: vec![],
            });
        }
        let text = std::str::from_utf8(data)
            .map_err(|_| IcaoError::UnsupportedFormat("DG14 non-UTF8 data".into()))?;
        let supported_protocols = text
            .split(|c: char| c == '\0' || c == '\n')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(Self {
            supported_protocols,
        })
    }
}
