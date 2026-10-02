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
use crate::record::{NistField, NistRecord, RecordType};

// ANSI/NIST-ITL separator bytes (NIST SP 500-290 e3, Section 7.3).
const FS: u8 = 0x1C; // File Separator -- record delimiter
const US: u8 = 0x1F; // Unit Separator -- field delimiter

/// A complete ANSI/NIST-ITL transaction containing one or more typed records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NistTransaction {
    pub records: Vec<NistRecord>,
}

impl NistTransaction {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn push_record(&mut self, record: NistRecord) {
        self.records.push(record);
    }

    /// Parse an ANSI/NIST binary stream into a transaction.
    ///
    /// Format per record:
    ///   `T.001:LEN US T.002:IDC [US T.NNN:value]* FS`
    ///
    /// Binary field values (images, BDB) are stored hex-encoded in the
    /// field value bytes, so no separator bytes appear inside values.
    pub fn parse(bytes: &[u8]) -> Result<Self, NistError> {
        let mut records = Vec::new();

        // Records are delimited by FS. Trailing empty chunk after final FS is ignored.
        for chunk in bytes.split(|&b| b == FS) {
            if chunk.is_empty() {
                continue;
            }
            records.push(parse_record(chunk)?);
        }

        if records.is_empty() {
            return Err(NistError::InvalidFormat(
                "transaction contains no records".to_string(),
            ));
        }

        Ok(Self { records })
    }

    /// Serialise the transaction to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for record in &self.records {
            out.extend_from_slice(&serialize_record(record));
        }
        out
    }
}

impl Default for NistTransaction {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_record(chunk: &[u8]) -> Result<NistRecord, NistError> {
    // chunk is everything between two FS bytes (or start/end of stream).
    // Fields are separated by US.
    let raw_fields: Vec<&[u8]> = chunk.split(|&b| b == US).collect();
    if raw_fields.is_empty() {
        return Err(NistError::InvalidFormat("empty record chunk".to_string()));
    }

    let mut record_type_byte: Option<u8> = None;
    let mut idc: u8 = 0;
    let mut fields: Vec<NistField> = Vec::new();

    for raw in raw_fields {
        if raw.is_empty() {
            continue;
        }
        let colon = raw
            .iter()
            .position(|&b| b == b':')
            .ok_or_else(|| NistError::InvalidFormat("field missing ':' separator".to_string()))?;
        let tag_str = std::str::from_utf8(&raw[..colon])
            .map_err(|e| NistError::InvalidFormat(format!("non-UTF-8 field tag: {e}")))?;
        let value = raw[colon + 1..].to_vec();

        let (rt_byte, field_num) = parse_tag(tag_str)?;

        if record_type_byte.is_none() {
            record_type_byte = Some(rt_byte);
        }

        match field_num {
            1 => {} // skip length field -- recomputed on serialize
            2 => {
                // IDC field
                let s = std::str::from_utf8(&value)
                    .map_err(|e| NistError::InvalidFormat(format!("non-UTF-8 IDC: {e}")))?;
                idc = s
                    .parse::<u8>()
                    .map_err(|e| NistError::InvalidFormat(format!("bad IDC value: {e}")))?;
            }
            _ => fields.push(NistField {
                tag: field_num,
                value,
            }),
        }
    }

    let rt_byte = record_type_byte
        .ok_or_else(|| NistError::InvalidFormat("record has no fields".to_string()))?;
    let record_type = RecordType::from_u8(rt_byte)?;

    Ok(NistRecord {
        record_type,
        idc,
        fields,
    })
}

/// Parse `"RRR.NNN"` -> `(record_type_u8, field_number)`.
fn parse_tag(tag: &str) -> Result<(u8, u32), NistError> {
    let dot = tag
        .find('.')
        .ok_or_else(|| NistError::InvalidFormat(format!("no '.' in field tag '{tag}'")))?;
    let rt: u8 = tag[..dot]
        .parse()
        .map_err(|_| NistError::InvalidFormat(format!("bad record type in tag '{tag}'")))?;
    let fnum: u32 = tag[dot + 1..]
        .parse()
        .map_err(|_| NistError::InvalidFormat(format!("bad field number in tag '{tag}'")))?;
    Ok((rt, fnum))
}

fn serialize_record(record: &NistRecord) -> Vec<u8> {
    let rt = record.record_type.as_u8();

    // Build all content except the length field: IDC field + user fields.
    let mut parts: Vec<Vec<u8>> = Vec::new();

    // T.002 = IDC (ASCII decimal)
    parts.push(make_field(rt, 2, record.idc.to_string().as_bytes()));

    // Remaining fields -- tags >= 3 by convention.
    for f in &record.fields {
        parts.push(make_field(rt, f.tag, &f.value));
    }

    // Join content fields with US.
    let content: Vec<u8> = parts.join(&[US][..]);

    // Compute total record length (self-inclusive) for T.001.
    // total = len("T.001:") + len(LEN_digits) + 1(US) + content.len() + 1(FS)
    let prefix = format!("{rt}.001:");
    let base = prefix.len() + 1 + content.len() + 1; // 1=US, 1=FS
    let total = resolve_length(base, &prefix);

    let len_field = format!("{prefix}{total}");
    let mut out = Vec::new();
    out.extend_from_slice(len_field.as_bytes());
    out.push(US);
    out.extend_from_slice(&content);
    out.push(FS);
    out
}

/// Build `"RT.TAG:value"` as a byte vec.
fn make_field(rt: u8, tag: u32, value: &[u8]) -> Vec<u8> {
    let mut v = format!("{rt}.{tag:03}:").into_bytes();
    v.extend_from_slice(value);
    v
}

/// Determine the self-consistent record length.
///
/// `base` = total bytes excluding the length-value digits themselves.
/// Iterates at most ~5 times (digit count of realistic record sizes is stable).
fn resolve_length(base: usize, prefix: &str) -> usize {
    let _ = prefix; // used only to document intent; already counted in base
    let mut digits = base.to_string().len();
    loop {
        let total = base + digits;
        let actual = total.to_string().len();
        if actual == digits {
            return total;
        }
        digits = actual;
    }
}
