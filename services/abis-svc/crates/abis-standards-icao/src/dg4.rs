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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrisEye {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisData {
    pub eye: IrisEye,
    pub image_data: Vec<u8>,
}

impl IrisData {
    /// Parse DG4 iris biometric data group.
    ///
    /// First byte encodes eye: 0x01 = Right, 0x02 = Left, per ISO/IEC 19794-6.
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.is_empty() {
            return Err(IcaoError::InvalidLength);
        }
        let eye = match data[0] {
            0x01 => IrisEye::Right,
            0x02 => IrisEye::Left,
            other => {
                return Err(IcaoError::UnsupportedFormat(format!(
                    "unknown iris eye code: 0x{other:02X}"
                )))
            }
        };
        Ok(Self {
            eye,
            image_data: data[1..].to_vec(),
        })
    }
}
