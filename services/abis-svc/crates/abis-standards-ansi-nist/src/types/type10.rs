use serde::{Deserialize, Serialize};

use crate::error::NistError;
use crate::record::NistField;

/// ANSI/NIST Type-10: Photographic body-part image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotographicBodyPart {
    pub isr: u8,
    pub hll: u16,
    pub vll: u16,
    pub slc: u8,
    pub image_data: Vec<u8>,
}

impl PhotographicBodyPart {
    pub fn parse(fields: &[NistField]) -> Result<Self, NistError> {
        let find = |tag: u32| -> Result<&NistField, NistError> {
            fields
                .iter()
                .find(|f| f.tag == tag)
                .ok_or(NistError::FieldMissing(tag))
        };

        let isr: u8 = find(3)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad ISR: {e}")))?;
        let hll: u16 = find(4)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad HLL: {e}")))?;
        let vll: u16 = find(5)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad VLL: {e}")))?;
        let slc: u8 = find(6)?
            .as_str()?
            .parse()
            .map_err(|e| NistError::InvalidFormat(format!("bad SLC: {e}")))?;
        let image_hex = find(999)?.as_str()?;
        let image_data = hex::decode(image_hex)
            .map_err(|e| NistError::InvalidFormat(format!("bad image hex: {e}")))?;

        Ok(Self {
            isr,
            hll,
            vll,
            slc,
            image_data,
        })
    }

    pub fn to_fields(&self) -> Vec<NistField> {
        vec![
            NistField::new(3, self.isr.to_string().as_bytes()),
            NistField::new(4, self.hll.to_string().as_bytes()),
            NistField::new(5, self.vll.to_string().as_bytes()),
            NistField::new(6, self.slc.to_string().as_bytes()),
            NistField::new(999, hex::encode(&self.image_data).as_bytes()),
        ]
    }
}
