//! Contact lens detection for iris PAD (presentation attack detection).
//!
//! References:
//!   Erdogmus, N., & Marcel, S. (2013). Spoofing 2D face recognition systems with 3D masks. BTAS.
//!   LivDet-Iris 2017.
//!
//! Detects three lens artefact classes: transparent, coloured, and patterned.

pub mod coloured;
pub mod detector;
pub mod patterned;
pub mod transparent;

pub use detector::{DetectError, LensDetector, LensKind, LensResult};
