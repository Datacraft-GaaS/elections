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

use serde::{Deserialize, Serialize};

use crate::{
    bdb::BiometricDataBlock, error::CbeffError, sb::SecurityBlock, sbh::StandardBiometricHeader,
};

/// Magic bytes that open every serialised CBEFF container.
pub const CBEFF_MAGIC: [u8; 4] = [0xCB, 0xEF, 0x00, 0x01];

/// A complete CBEFF container: SBH + BDB + optional SB.
///
/// Binary layout (all integers big-endian):
///
/// ```text
/// [4]  magic: 0xCB 0xEF 0x00 0x01
/// [N]  SBH (variable length)
/// [4]  bdb_length: u32
/// [M]  bdb_data
/// [1]  sb_present: 0 or 1
/// if sb_present:
///   [2]  algorithm_id: u16
///   [4]  sb_length: u32
///   [K]  sb_data
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CbeffContainer {
    pub sbh: StandardBiometricHeader,
    pub bdb: BiometricDataBlock,
    pub sb: Option<SecurityBlock>,
}

impl CbeffContainer {
    pub fn new(
        sbh: StandardBiometricHeader,
        bdb: BiometricDataBlock,
        sb: Option<SecurityBlock>,
    ) -> Self {
        Self { sbh, bdb, sb }
    }

    /// Parse a CBEFF container from raw bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, CbeffError> {
        if bytes.len() < CBEFF_MAGIC.len() || &bytes[..4] != CBEFF_MAGIC {
            return Err(CbeffError::InvalidMagic);
        }
        let mut pos = CBEFF_MAGIC.len();

        let (sbh, consumed) = StandardBiometricHeader::parse(&bytes[pos..])?;
        pos += consumed;

        if pos + 4 > bytes.len() {
            return Err(CbeffError::InvalidLength);
        }
        let bdb_length = u32::from_be_bytes(
            bytes[pos..pos + 4]
                .try_into()
                .map_err(|_| CbeffError::InvalidLength)?,
        ) as usize;
        pos += 4;

        if pos + bdb_length > bytes.len() {
            return Err(CbeffError::InvalidLength);
        }
        let bdb_data = bytes[pos..pos + bdb_length].to_vec();
        pos += bdb_length;

        let sb = if pos < bytes.len() {
            let sb_present = bytes[pos];
            pos += 1;
            if sb_present != 0 {
                if pos + 2 > bytes.len() {
                    return Err(CbeffError::InvalidLength);
                }
                let algorithm_id = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]);
                pos += 2;

                if pos + 4 > bytes.len() {
                    return Err(CbeffError::InvalidLength);
                }
                let sb_length = u32::from_be_bytes(
                    bytes[pos..pos + 4]
                        .try_into()
                        .map_err(|_| CbeffError::InvalidLength)?,
                ) as usize;
                pos += 4;

                if pos + sb_length > bytes.len() {
                    return Err(CbeffError::InvalidLength);
                }
                let sb_data = bytes[pos..pos + sb_length].to_vec();

                Some(SecurityBlock::new(algorithm_id, sb_data))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            sbh,
            bdb: BiometricDataBlock::new(bdb_data),
            sb,
        })
    }

    /// Serialise the container to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&CBEFF_MAGIC);
        self.sbh.serialize_into(&mut out);

        let bdb_len = self.bdb.data.len() as u32;
        out.extend_from_slice(&bdb_len.to_be_bytes());
        out.extend_from_slice(&self.bdb.data);

        match &self.sb {
            None => out.push(0),
            Some(sb) => {
                out.push(1);
                out.extend_from_slice(&sb.algorithm_id.to_be_bytes());
                let sb_len = sb.data.len() as u32;
                out.extend_from_slice(&sb_len.to_be_bytes());
                out.extend_from_slice(&sb.data);
            }
        }

        out
    }

    /// True if the BDB itself begins with CBEFF magic (nested container).
    pub fn is_nested(&self) -> bool {
        self.bdb.data.len() >= CBEFF_MAGIC.len()
            && &self.bdb.data[..CBEFF_MAGIC.len()] == CBEFF_MAGIC
    }

    /// Parse the nested CBEFF container stored inside the BDB.
    pub fn unwrap_nested(&self) -> Result<CbeffContainer, CbeffError> {
        if !self.is_nested() {
            return Err(CbeffError::MissingField(
                "bdb does not contain a nested CBEFF container".to_string(),
            ));
        }
        CbeffContainer::parse(&self.bdb.data).map_err(|e| CbeffError::NestedError(Box::new(e)))
    }
}
