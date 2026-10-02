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

/// ANSI/NIST-ITL record type identifiers (NIST SP 500-290 e3, Table 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecordType {
    Type1 = 1,
    Type2 = 2,
    Type3 = 3,
    Type4 = 4,
    Type5 = 5,
    Type6 = 6,
    Type7 = 7,
    Type8 = 8,
    Type9 = 9,
    Type10 = 10,
    Type11 = 11,
    Type12 = 12,
    Type13 = 13,
    Type14 = 14,
    Type15 = 15,
    Type16 = 16,
    Type17 = 17,
    Type18 = 18,
    Type19 = 19,
    Type20 = 20,
    Type21 = 21,
    Type22 = 22,
    Type98 = 98,
    Type99 = 99,
}

impl RecordType {
    /// Numeric value used in tag prefixes (e.g., `1` for `"1.001:"`).
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse from the numeric byte value in a tag prefix.
    pub fn from_u8(n: u8) -> Result<Self, NistError> {
        match n {
            1 => Ok(Self::Type1),
            2 => Ok(Self::Type2),
            3 => Ok(Self::Type3),
            4 => Ok(Self::Type4),
            5 => Ok(Self::Type5),
            6 => Ok(Self::Type6),
            7 => Ok(Self::Type7),
            8 => Ok(Self::Type8),
            9 => Ok(Self::Type9),
            10 => Ok(Self::Type10),
            11 => Ok(Self::Type11),
            12 => Ok(Self::Type12),
            13 => Ok(Self::Type13),
            14 => Ok(Self::Type14),
            15 => Ok(Self::Type15),
            16 => Ok(Self::Type16),
            17 => Ok(Self::Type17),
            18 => Ok(Self::Type18),
            19 => Ok(Self::Type19),
            20 => Ok(Self::Type20),
            21 => Ok(Self::Type21),
            22 => Ok(Self::Type22),
            98 => Ok(Self::Type98),
            99 => Ok(Self::Type99),
            _ => Err(NistError::UnknownRecordType(n)),
        }
    }
}

/// A single tagged field within an ANSI/NIST record.
///
/// `tag` is the field number (the `NNN` in `RRR.NNN`).
/// `value` contains the raw field bytes; binary data (images, BDB) is
/// stored hex-encoded so that separator bytes never appear in the value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NistField {
    pub tag: u32,
    pub value: Vec<u8>,
}

impl NistField {
    pub fn new(tag: u32, value: impl Into<Vec<u8>>) -> Self {
        Self {
            tag,
            value: value.into(),
        }
    }

    /// Convenience: decode value as UTF-8 text.
    pub fn as_str(&self) -> Result<&str, NistError> {
        std::str::from_utf8(&self.value)
            .map_err(|e| NistError::InvalidFormat(format!("field {}: {}", self.tag, e)))
    }
}

/// One logical record within an ANSI/NIST transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NistRecord {
    pub record_type: RecordType,
    /// Image designation character -- links records across a transaction.
    pub idc: u8,
    /// Type-specific fields (tag != 1 and tag != 2; those are managed by serialize/parse).
    pub fields: Vec<NistField>,
}

impl NistRecord {
    pub fn new(record_type: RecordType, idc: u8) -> Self {
        Self {
            record_type,
            idc,
            fields: Vec::new(),
        }
    }

    pub fn push_field(&mut self, tag: u32, value: impl Into<Vec<u8>>) {
        self.fields.push(NistField::new(tag, value));
    }

    /// Find the first field with the given tag.
    pub fn field(&self, tag: u32) -> Option<&NistField> {
        self.fields.iter().find(|f| f.tag == tag)
    }

    /// Find a required field, returning `FieldMissing` if absent.
    pub fn require_field(&self, tag: u32) -> Result<&NistField, NistError> {
        self.field(tag).ok_or(NistError::FieldMissing(tag))
    }
}
