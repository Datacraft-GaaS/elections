//! Motion blur estimation, multi-frame stacking, and iris identity tracking.

pub mod blur;
pub mod stack;
pub mod tracker;

pub use blur::{BlurEstimator, BlurResult};
pub use stack::FrameStacker;
pub use tracker::IrisTracker;
