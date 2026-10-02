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

use crate::error::NistError;
use crate::record::NistField;

/// ANSI/NIST Type-1: Transaction information record.
///
/// Mandatory header present in every transaction with IDC = 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionInfo {
    /// Type of transaction (e.g., `"CAR"`, `"SRE"`).
    pub tot: String,
    /// Date in `CCYYMMDD` format.
    pub dat: String,
    /// Priority (1-9).
    pub pry: u8,
    /// Destination agency identifier.
    pub dai: String,
    /// Originating agency identifier.
    pub ori: String,
    /// Transaction control number.
    pub tcn: String,
}

impl TransactionInfo {
    /// Parse from the fields of a Type-1 record.
    pub fn parse(fields: &[NistField]) -> Result<Self, NistError> {
        let find = |tag: u32| -> Result<&NistField, NistError> {
            fields
                .iter()
                .find(|f| f.tag == tag)
                .ok_or(NistError::FieldMissing(tag))
        };

        let tot = find(4)?.as_str()?.to_owned();
        let dat = find(5)?.as_str()?.to_owned();
        let pry_str = find(6)?.as_str()?;
        let pry: u8 = pry_str
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad PRY value: {e}")))?;
        let dai = find(7)?.as_str()?.to_owned();
        let ori = find(8)?.as_str()?.to_owned();
        let tcn = find(9)?.as_str()?.to_owned();

        Ok(Self {
            tot,
            dat,
            pry,
            dai,
            ori,
            tcn,
        })
    }

    /// Convert to the fields stored in a Type-1 `NistRecord` (tags 4-9).
    ///
    /// Tags 1 (length) and 2 (IDC) are managed by the transaction layer.
    pub fn to_fields(&self) -> Vec<NistField> {
        vec![
            NistField::new(3, b"0300".as_ref()), // VER
            NistField::new(4, self.tot.as_bytes()),
            NistField::new(5, self.dat.as_bytes()),
            NistField::new(6, self.pry.to_string().as_bytes()),
            NistField::new(7, self.dai.as_bytes()),
            NistField::new(8, self.ori.as_bytes()),
            NistField::new(9, self.tcn.as_bytes()),
        ]
    }
}
