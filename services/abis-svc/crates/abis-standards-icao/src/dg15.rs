// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
