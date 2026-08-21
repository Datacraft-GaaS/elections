//! Ridge counting between minutiae and K-plet descriptor matching.
//!
//! Proper Bozorth3-style ridge counting traverses the orientation field between
//! two minutiae and counts ridge crossings via zero-crossings perpendicular to
//! the path direction.
//!
//! Reference: C. I. Watson, "NIST Special Database 4, Fingerprint Database,"
//! NIST, 1992; C. Watson et al., "User's Guide to NIST Biometric Image Software
//! (NBIS)," NISTIR 7392, 2007, describing the Bozorth3 algorithm.

pub mod count;
pub mod kplet;
pub mod matcher;

pub use count::{count_ridges, OrientationField};
pub use kplet::{KPlet, KPletDescriptor};
pub use matcher::{KPletMatcher, RidgeCountMatcher};
