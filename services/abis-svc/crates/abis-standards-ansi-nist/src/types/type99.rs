use serde::{Deserialize, Serialize};

use crate::error::NistError;
use crate::record::NistField;

/// ANSI/NIST Type-99: CBEFF biometric data record.
///
/// Wraps a CBEFF container as a native ANSI/NIST record.
/// The BDB is hex-encoded in the serialised form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CbeffBiometric {
    /// Format owner registered with IBIA.
    pub format_owner: u16,
    /// Format type within the owner's namespace.
    pub format_type: u16,
    /// Raw biometric data block bytes.
    pub bdb: Vec<u8>,
}

impl CbeffBiometric {
    pub fn parse(fields: &[NistField]) -> Result<Self, NistError> {
        let find = |tag: u32| -> Result<&NistField, NistError> {
            fields
                .iter()
                .find(|f| f.tag == tag)
                .ok_or(NistError::FieldMissing(tag))
        };

        let format_owner: u16 = find(3)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad format_owner: {e}")))?;
        let format_type: u16 = find(4)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad format_type: {e}")))?;
        let bdb_hex = find(999)?.as_str()?;
        let bdb = hex::decode(bdb_hex)
            .map_err(|e| NistError::InvalidFormat(format!("bad BDB hex: {e}")))?;

        Ok(Self {
            format_owner,
            format_type,
            bdb,
        })
    }

    pub fn to_fields(&self) -> Vec<NistField> {
        vec![
            NistField::new(3, self.format_owner.to_string().as_bytes()),
            NistField::new(4, self.format_type.to_string().as_bytes()),
            NistField::new(999, hex::encode(&self.bdb).as_bytes()),
        ]
    }
}
