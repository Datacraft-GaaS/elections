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
