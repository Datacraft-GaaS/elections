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

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{error::EbtsError, tot::TypeOfTransaction};

/// A single additional record appended to an EBTS transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EbtsRecord {
    /// Image designation character.
    pub idc: u8,
    /// ANSI/NIST record type (e.g. 4 for fingerprint image).
    pub record_type: u8,
    /// Raw record content bytes (no length prefix, no separator).
    pub data: Vec<u8>,
}

/// A complete FBI EBTS transaction.
///
/// Binary layout:
///
/// ```text
/// Type-1 header record:
///   [1]  record_type = 1
///   [1]  idc = 0
///   [2]  header_len: u16 (be), length of content bytes
///   [N]  content: "TOT:<tot>\x1fDAI:<dai>\x1fORI:<ori>\x1fTCN:<tcn>" (ASCII)
///
/// Additional records (one per EbtsRecord):
///   [1]  record_type
///   [1]  idc
///   [4]  data_len: u32 (be)
///   [M]  data (hex-encoded to avoid separator ambiguity)
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EbtsTransaction {
    pub tot: TypeOfTransaction,
    pub dai: String,
    pub ori: String,
    pub tcn: String,
    pub records: Vec<EbtsRecord>,
}

const US: u8 = 0x1F;

impl EbtsTransaction {
    /// Construct a new transaction with no additional records.
    pub fn build(
        tot: TypeOfTransaction,
        dai: impl Into<String>,
        ori: impl Into<String>,
        tcn: impl Into<String>,
    ) -> Self {
        Self {
            tot,
            dai: dai.into(),
            ori: ori.into(),
            tcn: tcn.into(),
            records: Vec::new(),
        }
    }

    /// Append an additional record.
    pub fn add_record(&mut self, idc: u8, record_type: u8, data: Vec<u8>) {
        self.records.push(EbtsRecord {
            idc,
            record_type,
            data,
        });
    }

    /// Serialise to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // Type-1 header
        let content = format!(
            "TOT:{}{US}DAI:{}{US}ORI:{}{US}TCN:{}",
            self.tot,
            self.dai,
            self.ori,
            self.tcn,
            US = US as char,
        );
        let content_bytes = content.as_bytes();
        let header_len = content_bytes.len() as u16;

        out.push(1_u8); // record_type
        out.push(0_u8); // idc
        out.extend_from_slice(&header_len.to_be_bytes());
        out.extend_from_slice(content_bytes);

        // Additional records
        for rec in &self.records {
            out.push(rec.record_type);
            out.push(rec.idc);
            // Hex-encode data to avoid ambiguity with separator bytes.
            let hex = hex_encode(&rec.data);
            let data_len = hex.len() as u32;
            out.extend_from_slice(&data_len.to_be_bytes());
            out.extend_from_slice(hex.as_bytes());
        }

        out
    }

    /// Parse a transaction from bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, EbtsError> {
        let mut pos = 0;

        // Type-1 header
        if pos + 4 > bytes.len() {
            return Err(EbtsError::InvalidFormat("truncated header".to_string()));
        }
        let rt = bytes[pos];
        if rt != 1 {
            return Err(EbtsError::InvalidFormat(format!(
                "expected record_type=1, got {rt}"
            )));
        }
        let _idc = bytes[pos + 1]; // always 0
        let header_len = u16::from_be_bytes([bytes[pos + 2], bytes[pos + 3]]) as usize;
        pos += 4;

        if pos + header_len > bytes.len() {
            return Err(EbtsError::InvalidFormat(
                "header content truncated".to_string(),
            ));
        }
        let content = std::str::from_utf8(&bytes[pos..pos + header_len])
            .map_err(|e| EbtsError::InvalidFormat(format!("non-UTF-8 header: {e}")))?;
        pos += header_len;

        let (tot, dai, ori, tcn) = parse_header_content(content)?;

        // Additional records
        let mut records = Vec::new();
        while pos < bytes.len() {
            if pos + 2 > bytes.len() {
                return Err(EbtsError::InvalidFormat(
                    "truncated record header".to_string(),
                ));
            }
            let record_type = bytes[pos];
            let idc = bytes[pos + 1];
            pos += 2;

            if pos + 4 > bytes.len() {
                return Err(EbtsError::InvalidFormat(
                    "truncated record data_len".to_string(),
                ));
            }
            let data_len = u32::from_be_bytes(
                bytes[pos..pos + 4]
                    .try_into()
                    .map_err(|_| EbtsError::InvalidFormat("bad data_len".to_string()))?,
            ) as usize;
            pos += 4;

            if pos + data_len > bytes.len() {
                return Err(EbtsError::InvalidFormat(
                    "record data truncated".to_string(),
                ));
            }
            let hex_str = std::str::from_utf8(&bytes[pos..pos + data_len])
                .map_err(|e| EbtsError::InvalidFormat(format!("non-UTF-8 data: {e}")))?;
            let data = hex_decode(hex_str)?;
            pos += data_len;

            records.push(EbtsRecord {
                idc,
                record_type,
                data,
            });
        }

        Ok(Self {
            tot,
            dai,
            ori,
            tcn,
            records,
        })
    }
}

fn parse_header_content(
    content: &str,
) -> Result<(TypeOfTransaction, String, String, String), EbtsError> {
    let mut tot = None;
    let mut dai = None;
    let mut ori = None;
    let mut tcn = None;

    for part in content.split('\x1f') {
        if let Some(v) = part.strip_prefix("TOT:") {
            tot = Some(TypeOfTransaction::from_str(v)?);
        } else if let Some(v) = part.strip_prefix("DAI:") {
            dai = Some(v.to_owned());
        } else if let Some(v) = part.strip_prefix("ORI:") {
            ori = Some(v.to_owned());
        } else if let Some(v) = part.strip_prefix("TCN:") {
            tcn = Some(v.to_owned());
        }
    }

    Ok((
        tot.ok_or_else(|| EbtsError::MissingField("TOT".to_string()))?,
        dai.ok_or_else(|| EbtsError::MissingField("DAI".to_string()))?,
        ori.ok_or_else(|| EbtsError::MissingField("ORI".to_string()))?,
        tcn.ok_or_else(|| EbtsError::MissingField("TCN".to_string()))?,
    ))
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, EbtsError> {
    if s.len() % 2 != 0 {
        return Err(EbtsError::InvalidFormat(
            "odd-length hex string".to_string(),
        ));
    }
    (0..s.len() / 2)
        .map(|i| {
            u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|e| EbtsError::InvalidFormat(format!("bad hex byte at {i}: {e}")))
        })
        .collect()
}
