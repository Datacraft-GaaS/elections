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

/// A single minutiae point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinutiaePoint {
    pub x: u16,
    pub y: u16,
    pub theta: u16,
    pub quality: u8,
    pub mtype: u8,
}

/// ANSI/NIST Type-9: Extended feature set / minutiae record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Minutiae {
    /// Impression type.
    pub imp: u8,
    /// Finger minutiae format identifier.
    pub fmt: String,
    pub minutiae: Vec<MinutiaePoint>,
}

impl Minutiae {
    pub fn parse(fields: &[NistField]) -> Result<Self, NistError> {
        let find = |tag: u32| -> Result<&NistField, NistError> {
            fields
                .iter()
                .find(|f| f.tag == tag)
                .ok_or(NistError::FieldMissing(tag))
        };

        let imp: u8 = find(3)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad IMP: {e}")))?;
        let fmt = find(4)?.as_str()?.to_owned();
        let min_str = find(5)?.as_str()?;
        let minutiae = parse_minutiae(min_str)?;

        Ok(Self { imp, fmt, minutiae })
    }

    pub fn to_fields(&self) -> Vec<NistField> {
        vec![
            NistField::new(3, self.imp.to_string().as_bytes()),
            NistField::new(4, self.fmt.as_bytes()),
            NistField::new(5, encode_minutiae(&self.minutiae).as_bytes()),
        ]
    }
}

/// Encode minutiae as `"X,Y,THETA,QUALITY,TYPE;..."`.
fn encode_minutiae(pts: &[MinutiaePoint]) -> String {
    pts.iter()
        .map(|p| format!("{},{},{},{},{}", p.x, p.y, p.theta, p.quality, p.mtype))
        .collect::<Vec<_>>()
        .join(";")
}

/// Decode `"X,Y,THETA,QUALITY,TYPE;..."`.
fn parse_minutiae(s: &str) -> Result<Vec<MinutiaePoint>, NistError> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    s.split(';')
        .map(|entry| {
            let parts: Vec<&str> = entry.split(',').collect();
            if parts.len() != 5 {
                return Err(NistError::InvalidFormat(format!(
                    "minutiae entry must have 5 comma-separated values, got: '{entry}'"
                )));
            }
            let parse_u16 = |s: &str, name: &str| -> Result<u16, NistError> {
                s.parse()
                    .map_err(|e| NistError::InvalidFormat(format!("bad {name}: {e}")))
            };
            let parse_u8 = |s: &str, name: &str| -> Result<u8, NistError> {
                s.parse()
                    .map_err(|e| NistError::InvalidFormat(format!("bad {name}: {e}")))
            };
            Ok(MinutiaePoint {
                x: parse_u16(parts[0], "x")?,
                y: parse_u16(parts[1], "y")?,
                theta: parse_u16(parts[2], "theta")?,
                quality: parse_u8(parts[3], "quality")?,
                mtype: parse_u8(parts[4], "mtype")?,
            })
        })
        .collect()
}
