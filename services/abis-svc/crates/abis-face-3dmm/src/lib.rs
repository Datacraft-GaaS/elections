//! 3D Morphable Model (3DMM) interface for face reconstruction and PAD.
//!
//! References:
//! - Blanz, V. & Vetter, T. (1999). "A morphable model for the synthesis
//!   of 3D faces." SIGGRAPH 1999. (original 3DMM formulation)
//! - Basel Face Model (BFM): Paysan et al. (2009). "A 3D Face Model for Pose
//!   and Illumination Invariant Face Recognition." AVSS 2009.
//! - Deng, Y. et al. (2019). "Accurate 3D Face Reconstruction with Weakly-
//!   Supervised Learning." CVPRW 2019. (BFM parameter shapes)
#![warn(missing_docs)]

pub mod bfm;
pub mod spoof;

pub use bfm::{Fit3dmm, Pose3d, Camera3d, Model3dmm, DmmError};
pub use spoof::{SpoofDecision, SpoofDetector};
