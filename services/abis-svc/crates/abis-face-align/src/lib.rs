//! Face alignment: similarity transform mapping 5 detected landmarks to the
//! ArcFace 112x112 canonical template.
//!
//! Reference:
//! - Deng, J. et al. (2019). "ArcFace: Additive Angular Margin Loss for Deep
//!   Face Recognition." CVPR 2019. Appendix A defines the 112x112 canonical
//!   landmark positions used here.
//! - Cootes, T. F. et al. (1992). "Active Shape Models." Springer. (68-point
//!   landmark convention.)
#![warn(missing_docs)]

pub mod canonical;
pub mod landmarks;
pub mod transform;
pub mod warp;

pub use canonical::CANONICAL_5;
pub use landmarks::{FiveLandmarks, Landmark, Landmarks68};
pub use transform::{AlignError, SimilarityTransform};
pub use warp::{warp_bilinear, AlignedFace};

/// Output size of the canonical aligned face image (ArcFace convention).
pub const ALIGN_SIZE: u32 = 112;
