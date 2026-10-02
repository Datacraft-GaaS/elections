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

/// ANSI/NIST Type-2: User-defined text record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDefinedText {
    /// Raw user-defined field bytes (tag 3).
    pub udf: Vec<u8>,
}

impl UserDefinedText {
    pub fn parse(fields: &[NistField]) -> Result<Self, NistError> {
        let f = fields
            .iter()
            .find(|f| f.tag == 3)
            .ok_or(NistError::FieldMissing(3))?;
        Ok(Self {
            udf: f.value.clone(),
        })
    }

    pub fn to_fields(&self) -> Vec<NistField> {
        vec![NistField::new(3, self.udf.clone())]
    }
}
