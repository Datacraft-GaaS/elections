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
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSecurityObject {
    pub hash_algorithm: String,
    /// (data_group_number, hash_bytes) pairs.
    pub data_group_hashes: Vec<(u8, Vec<u8>)>,
    pub signed_attrs: Vec<u8>,
}

impl DocumentSecurityObject {
    /// Parse a SOD structure. The minimal encoding used here is:
    ///   - 1 byte: algorithm name length
    ///   - N bytes: algorithm name (UTF-8)
    ///   - 1 byte: number of hash entries
    ///   - for each entry: 1 byte DG number, 1 byte hash length, N bytes hash
    ///   - remainder: signed_attrs
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.is_empty() {
            return Err(IcaoError::InvalidLength);
        }
        let mut pos = 0;
        let alg_len = data[pos] as usize;
        pos += 1;
        if data.len() < pos + alg_len {
            return Err(IcaoError::InvalidLength);
        }
        let hash_algorithm = std::str::from_utf8(&data[pos..pos + alg_len])
            .map_err(|_| IcaoError::UnsupportedFormat("SOD algorithm not UTF-8".into()))?
            .to_string();
        pos += alg_len;

        if data.len() <= pos {
            return Err(IcaoError::InvalidLength);
        }
        let entry_count = data[pos] as usize;
        pos += 1;

        let mut data_group_hashes = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            if data.len() < pos + 2 {
                return Err(IcaoError::InvalidLength);
            }
            let dg_num = data[pos];
            let hash_len = data[pos + 1] as usize;
            pos += 2;
            if data.len() < pos + hash_len {
                return Err(IcaoError::InvalidLength);
            }
            let hash = data[pos..pos + hash_len].to_vec();
            pos += hash_len;
            data_group_hashes.push((dg_num, hash));
        }

        let signed_attrs = data[pos..].to_vec();
        Ok(Self {
            hash_algorithm,
            data_group_hashes,
            signed_attrs,
        })
    }

    /// Verify passive authentication: hash dg_data with SHA-256 and compare to stored value.
    pub fn verify_passive_auth(&self, dg_number: u8, dg_data: &[u8]) -> Result<bool, IcaoError> {
        let computed: Vec<u8> = Sha256::digest(dg_data).to_vec();
        for (num, stored_hash) in &self.data_group_hashes {
            if *num == dg_number {
                return Ok(computed == *stored_hash);
            }
        }
        // DG not present in SOD; treat as unverified.
        Ok(false)
    }

    /// Encode this SOD to the minimal wire format understood by `parse`.
    pub fn encode(&self) -> Vec<u8> {
        let alg = self.hash_algorithm.as_bytes();
        let mut out = vec![alg.len() as u8];
        out.extend_from_slice(alg);
        out.push(self.data_group_hashes.len() as u8);
        for (num, hash) in &self.data_group_hashes {
            out.push(*num);
            out.push(hash.len() as u8);
            out.extend_from_slice(hash);
        }
        out.extend_from_slice(&self.signed_attrs);
        out
    }
}
