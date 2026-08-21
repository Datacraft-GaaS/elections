//! Privacy-preserving face de-identification.
//!
//! References:
//! - Sweeney, L. (2002). "k-anonymity: A model for protecting privacy."
//!   IJUFKS 10(5):557-570. (k-anonymity formal definition)
//! - Dwork, C. (2006). "Differential Privacy." ICALP 2006. (DP formal
//!   definition; Gaussian mechanism used here)
//! - Gross, R. et al. (2009). "Face de-identification." Protection of
//!   Privacy in Surveillance Systems. Springer. (composite-face approach)

pub mod kanon;
pub mod dp;
pub mod blur;

pub use kanon::{KAnonDeident, KAnonError};
pub use dp::{DpEmbeddingNoise, DpError};
pub use blur::{blur_image, pixelate_image};
