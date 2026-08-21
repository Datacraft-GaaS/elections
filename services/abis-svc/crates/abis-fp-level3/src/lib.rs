//! Level-3 fingerprint feature extraction and matching.
//!
//! Level-3 features include sweat pores, ridge edge contours, incipient ridges,
//! and require image resolution >= 500 ppi for reliable extraction.
//!
//! Reference: A. K. Jain, Y. Chen, M. Demirkus, "Pores and Ridges: High-Resolution
//! Fingerprint Matching Using Level 3 Features," IEEE Trans. Pattern Anal. Mach.
//! Intell., vol. 29, no. 1, pp. 15-27, Jan. 2007.

pub mod incipient;
pub mod matcher;
pub mod pores;
pub mod quality;
pub mod ridge_edge;

pub use incipient::IncipientRidge;
pub use matcher::Level3Matcher;
pub use pores::Pore;
pub use quality::{Level3Quality, Level3QualityError};
pub use ridge_edge::EdgePoint;
