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

/// Quality record as defined in ISO/IEC 19785-1 clause 7.8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityRecord {
    pub algorithm_vendor: u16,
    pub algorithm_product: u16,
    pub quality_score: u8,
}

impl QualityRecord {
    /// Size in bytes when serialised.
    pub const WIRE_SIZE: usize = 5; // u16 + u16 + u8

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::WIRE_SIZE {
            return None;
        }
        Some(Self {
            algorithm_vendor: u16::from_be_bytes([buf[0], buf[1]]),
            algorithm_product: u16::from_be_bytes([buf[2], buf[3]]),
            quality_score: buf[4],
        })
    }

    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.algorithm_vendor.to_be_bytes());
        out.extend_from_slice(&self.algorithm_product.to_be_bytes());
        out.push(self.quality_score);
    }
}

/// Standard biometric header (ISO/IEC 19785-1 clause 7).
///
/// All multi-byte integers are big-endian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandardBiometricHeader {
    pub format_owner: u16,
    pub format_type: u16,
    /// Eight ASCII bytes: YYYYMMDD.
    pub creation_date: [u8; 8],
    /// Optional 16-byte validity period (start + end, YYYYMMDD each).
    pub validity_period: Option<[u8; 16]>,
    /// Biometric type flags (ISO/IEC 19785-1 Table 3).
    pub biometric_type: u32,
    pub biometric_subtype: u8,
    pub product_id: u16,
    pub capture_device_id: u16,
    pub quality_records: Vec<QualityRecord>,
}

impl StandardBiometricHeader {
    /// Parse an SBH from the start of `buf`.
    /// Returns the parsed SBH and the number of bytes consumed.
    pub fn parse(buf: &[u8]) -> Result<(Self, usize), crate::error::CbeffError> {
        use crate::error::CbeffError;

        let mut pos = 0;

        let read_u16 = |buf: &[u8], pos: &mut usize| -> Result<u16, CbeffError> {
            if *pos + 2 > buf.len() {
                return Err(CbeffError::InvalidLength);
            }
            let v = u16::from_be_bytes([buf[*pos], buf[*pos + 1]]);
            *pos += 2;
            Ok(v)
        };

        let format_owner = read_u16(buf, &mut pos)?;
        let format_type = read_u16(buf, &mut pos)?;

        if pos + 8 > buf.len() {
            return Err(CbeffError::InvalidLength);
        }
        let creation_date: [u8; 8] = buf[pos..pos + 8]
            .try_into()
            .map_err(|_| CbeffError::InvalidLength)?;
        pos += 8;

        // validity_period presence flag
        if pos >= buf.len() {
            return Err(CbeffError::InvalidLength);
        }
        let vp_flag = buf[pos];
        pos += 1;
        let validity_period = if vp_flag != 0 {
            if pos + 16 > buf.len() {
                return Err(CbeffError::InvalidLength);
            }
            let vp: [u8; 16] = buf[pos..pos + 16]
                .try_into()
                .map_err(|_| CbeffError::InvalidLength)?;
            pos += 16;
            Some(vp)
        } else {
            None
        };

        if pos + 4 > buf.len() {
            return Err(CbeffError::InvalidLength);
        }
        let biometric_type = u32::from_be_bytes(
            buf[pos..pos + 4]
                .try_into()
                .map_err(|_| CbeffError::InvalidLength)?,
        );
        pos += 4;

        if pos >= buf.len() {
            return Err(CbeffError::InvalidLength);
        }
        let biometric_subtype = buf[pos];
        pos += 1;

        let product_id = read_u16(buf, &mut pos)?;
        let capture_device_id = read_u16(buf, &mut pos)?;

        // quality record count
        let quality_count = read_u16(buf, &mut pos)? as usize;
        let mut quality_records = Vec::with_capacity(quality_count);
        for _ in 0..quality_count {
            let qr = QualityRecord::parse(&buf[pos..]).ok_or(CbeffError::InvalidLength)?;
            pos += QualityRecord::WIRE_SIZE;
            quality_records.push(qr);
        }

        Ok((
            Self {
                format_owner,
                format_type,
                creation_date,
                validity_period,
                biometric_type,
                biometric_subtype,
                product_id,
                capture_device_id,
                quality_records,
            },
            pos,
        ))
    }

    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.format_owner.to_be_bytes());
        out.extend_from_slice(&self.format_type.to_be_bytes());
        out.extend_from_slice(&self.creation_date);

        if let Some(vp) = &self.validity_period {
            out.push(1);
            out.extend_from_slice(vp);
        } else {
            out.push(0);
        }

        out.extend_from_slice(&self.biometric_type.to_be_bytes());
        out.push(self.biometric_subtype);
        out.extend_from_slice(&self.product_id.to_be_bytes());
        out.extend_from_slice(&self.capture_device_id.to_be_bytes());

        let count = self.quality_records.len() as u16;
        out.extend_from_slice(&count.to_be_bytes());
        for qr in &self.quality_records {
            qr.serialize_into(out);
        }
    }
}
