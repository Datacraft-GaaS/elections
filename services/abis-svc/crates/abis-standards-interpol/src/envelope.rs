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

use crate::InterpolError;
use serde::{Deserialize, Serialize};

/// Three-letter INTERPOL National Central Bureau code (e.g. "KEN", "INT").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NcbCode(pub String);

impl NcbCode {
    pub fn new(code: &str) -> Result<Self, InterpolError> {
        if code.len() != 3 {
            return Err(InterpolError::InvalidNcbCode(format!(
                "NCB code must be 3 characters, got {}: {}",
                code.len(),
                code
            )));
        }
        if !code.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(InterpolError::InvalidNcbCode(format!(
                "NCB code must be ASCII alpha, got: {code}"
            )));
        }
        Ok(Self(code.to_uppercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessagePurpose {
    CriminalSearch,
    PersonSearch,
    LostDocumentSearch,
    StolenVehicleSearch,
}

/// Expected response time in hours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseWindow {
    /// Immediate response required (0 hours).
    Immediate,
    /// Standard 24-hour response.
    Standard,
    /// Relaxed 72-hour response.
    Relaxed,
}

impl ResponseWindow {
    pub fn hours(&self) -> u32 {
        match self {
            ResponseWindow::Immediate => 0,
            ResponseWindow::Standard => 24,
            ResponseWindow::Relaxed => 72,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeRecord {
    pub record_type: u8,
    pub idc: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolEnvelope {
    pub ncb_sender: NcbCode,
    pub ncb_receiver: NcbCode,
    pub message_id: String,
    pub purpose: MessagePurpose,
    pub response_window: ResponseWindow,
    pub nist_type1_idc: u8,
    pub sender_ref: String,
    pub records: Vec<EnvelopeRecord>,
}

impl InterpolEnvelope {
    pub fn build(
        ncb_sender: NcbCode,
        ncb_receiver: NcbCode,
        purpose: MessagePurpose,
        response_window: ResponseWindow,
    ) -> Result<Self, InterpolError> {
        Ok(Self {
            ncb_sender,
            ncb_receiver,
            message_id: uuid_string(),
            purpose,
            response_window,
            nist_type1_idc: 0,
            sender_ref: String::new(),
            records: Vec::new(),
        })
    }

    pub fn add_record(&mut self, record_type: u8, idc: u8, data: Vec<u8>) {
        self.records.push(EnvelopeRecord {
            record_type,
            idc,
            data,
        });
    }

    /// Serialize to a compact binary format for transmission.
    ///
    /// Format: JSON bytes, length-prefixed with 4-byte big-endian u32.
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_vec(self).unwrap_or_default();
        let len = json.len() as u32;
        let mut out = len.to_be_bytes().to_vec();
        out.extend_from_slice(&json);
        out
    }

    /// Parse an envelope from bytes produced by `serialize`.
    pub fn parse(bytes: &[u8]) -> Result<Self, InterpolError> {
        if bytes.len() < 4 {
            return Err(InterpolError::InvalidFormat(
                "envelope too short for length prefix".into(),
            ));
        }
        let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        if bytes.len() < 4 + len {
            return Err(InterpolError::InvalidFormat(format!(
                "envelope truncated: expected {} bytes of payload, got {}",
                len,
                bytes.len() - 4
            )));
        }
        serde_json::from_slice(&bytes[4..4 + len])
            .map_err(|e| InterpolError::InvalidFormat(e.to_string()))
    }
}

fn uuid_string() -> String {
    // Simple time-based placeholder - avoids pulling in uuid dep for this crate.
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("eel-msg-{nanos:010}")
}
