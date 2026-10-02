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

/// TD3 Machine Readable Zone - 88 characters per ICAO 9303 Part 4.
#[derive(Debug, Clone)]
pub struct MachineReadableZone {
    pub raw: [u8; 88],
}

impl MachineReadableZone {
    /// Parse a TD3 MRZ from raw bytes. Must be exactly 88 bytes.
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.len() != 88 {
            return Err(IcaoError::InvalidMrz(format!(
                "expected 88 bytes, got {}",
                data.len()
            )));
        }
        let mut raw = [0u8; 88];
        raw.copy_from_slice(data);
        Ok(Self { raw })
    }

    fn field_str(&self, start: usize, end: usize) -> Result<&str, IcaoError> {
        std::str::from_utf8(&self.raw[start..end])
            .map_err(|_| IcaoError::InvalidMrz("non-UTF8 in MRZ field".into()))
    }

    fn trim_filler(s: &str) -> &str {
        s.trim_end_matches('<')
    }

    /// Family name (primary identifier, positions 5-43, first segment before "<<").
    pub fn surname(&self) -> Result<&str, IcaoError> {
        let primary = self.field_str(5, 44)?;
        let name = primary.split("<<").next().unwrap_or(primary);
        Ok(Self::trim_filler(name))
    }

    /// Given names (primary identifier, positions 5-43, after "<<").
    pub fn given_names(&self) -> Result<&str, IcaoError> {
        let primary = self.field_str(5, 44)?;
        let names = primary.split("<<").nth(1).unwrap_or("");
        Ok(Self::trim_filler(names))
    }

    /// Document number (secondary line positions 0-8, trimmed of filler).
    pub fn document_number(&self) -> Result<&str, IcaoError> {
        let field = self.field_str(44, 53)?;
        Ok(Self::trim_filler(field))
    }

    /// Nationality (positions 54-56 in secondary line).
    pub fn nationality(&self) -> Result<&str, IcaoError> {
        let field = self.field_str(54, 57)?;
        Ok(Self::trim_filler(field))
    }

    /// Date of birth YYMMDD (positions 57-62 in secondary line).
    pub fn birth_date(&self) -> Result<&str, IcaoError> {
        self.field_str(57, 63)
    }

    /// Date of expiry YYMMDD (positions 65-70 in secondary line).
    pub fn expiry_date(&self) -> Result<&str, IcaoError> {
        self.field_str(65, 71)
    }

    /// Personal number / optional data (positions 72-85 in secondary line).
    pub fn personal_number(&self) -> Result<&str, IcaoError> {
        let field = self.field_str(72, 86)?;
        Ok(Self::trim_filler(field))
    }
}

/// Compute ICAO 9303 check digit using weights 7, 3, 1 repeating.
///
/// Characters: '0'-'9' -> 0-9, 'A'-'Z' -> 10-35, '<' -> 0.
pub fn check_digit(field: &[u8]) -> u8 {
    let weights = [7u32, 3, 1];
    let sum: u32 = field
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            let v = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'A'..=b'Z' => (b - b'A' + 10) as u32,
                b'<' => 0,
                _ => 0,
            };
            v * weights[i % 3]
        })
        .sum();
    (sum % 10) as u8
}
