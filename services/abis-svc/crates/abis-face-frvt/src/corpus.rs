//! FRVT corpus loader types.
//!
//! Reference: NIST FRVT Ongoing protocol (frvt.nist.gov/frvt1N).
//! Image set categories: Visa, Mugshot, Wild, Kiosk.

use serde::{Deserialize, Serialize};

/// FRVT image source category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrvtImageSet {
    Visa,
    Mugshot,
    Wild,
    Kiosk,
}

/// Biological sex (ISO 5218 subset).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

/// Fitzpatrick skin tone scale (I-VI).
/// Reference: Fitzpatrick, T. B. (1988). "The validity and practicality of
/// sun-reactive skin types I through VI." Arch Dermatol 124(6):869-871.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkinTone {
    I,
    II,
    III,
    IV,
    V,
    VI,
    Unknown,
}

/// A single face image in the FRVT corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrvtImage {
    /// Unique image identifier.
    pub id: String,
    /// Subject (identity) identifier.
    pub subject_id: String,
    /// Source category.
    pub image_set: FrvtImageSet,
    /// Age at time of capture.
    pub age: Option<u8>,
    pub sex: Sex,
    pub skin_tone: SkinTone,
    /// L2-normalised 512-d embedding.
    pub embedding: Vec<f32>,
    /// Quality score [0.0, 1.0].
    pub quality: f32,
}

impl FrvtImage {
    pub fn new(
        id: impl Into<String>,
        subject_id: impl Into<String>,
        image_set: FrvtImageSet,
        embedding: Vec<f32>,
        quality: f32,
    ) -> Self {
        Self {
            id: id.into(),
            subject_id: subject_id.into(),
            image_set,
            age: None,
            sex: Sex::Unknown,
            skin_tone: SkinTone::Unknown,
            embedding,
            quality,
        }
    }

    pub fn with_demographics(mut self, age: u8, sex: Sex, skin_tone: SkinTone) -> Self {
        self.age = Some(age);
        self.sex = sex;
        self.skin_tone = skin_tone;
        self
    }
}

/// A loaded FRVT corpus.
#[derive(Debug, Clone, Default)]
pub struct FrvtCorpus {
    pub images: Vec<FrvtImage>,
}

impl FrvtCorpus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, img: FrvtImage) {
        self.images.push(img);
    }

    /// All genuine pairs: same subject_id, different image id.
    pub fn genuine_pairs(&self) -> Vec<(&FrvtImage, &FrvtImage)> {
        let mut pairs = Vec::new();
        let n = self.images.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if self.images[i].subject_id == self.images[j].subject_id {
                    pairs.push((&self.images[i], &self.images[j]));
                }
            }
        }
        pairs
    }

    /// All impostor pairs: different subject_id.
    pub fn impostor_pairs(&self) -> Vec<(&FrvtImage, &FrvtImage)> {
        let mut pairs = Vec::new();
        let n = self.images.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if self.images[i].subject_id != self.images[j].subject_id {
                    pairs.push((&self.images[i], &self.images[j]));
                }
            }
        }
        pairs
    }
}
