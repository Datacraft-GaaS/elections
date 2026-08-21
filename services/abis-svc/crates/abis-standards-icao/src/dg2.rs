use crate::IcaoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageFormat {
    Jpeg,
    Jpeg2000,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceImageData {
    pub image_format: ImageFormat,
    pub image_data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl FaceImageData {
    /// Parse DG2 facial image data. Detects format by magic bytes.
    pub fn parse(data: &[u8]) -> Result<Self, IcaoError> {
        if data.len() < 4 {
            return Err(IcaoError::InvalidLength);
        }
        let image_format = if data[0] == 0xFF && data[1] == 0xD8 {
            ImageFormat::Jpeg
        } else if data[0] == 0xFF && data[1] == 0x4F {
            ImageFormat::Jpeg2000
        } else if data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x00 && data[3] == 0x0C {
            // JP2 file format box
            ImageFormat::Jpeg2000
        } else {
            return Err(IcaoError::UnsupportedFormat(format!(
                "unknown image magic: {:02X} {:02X}",
                data[0], data[1]
            )));
        };
        Ok(Self {
            image_format,
            image_data: data.to_vec(),
            width: 0,
            height: 0,
        })
    }
}
