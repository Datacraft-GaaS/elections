//! Multi-frame face tracking and quality-weighted embedding fusion.
//!
//! References:
//! - Bewley, A. et al. (2016). "Simple Online and Realtime Tracking." ICIP 2016.
//!   (IoU-based assignment for short-term tracking, the SORT algorithm)
//! - Best, S. & Best, N. (2021). "Quality-weighted averaging of face
//!   embeddings across video frames." FG 2021. (quality-weighted fusion)

pub mod track;
pub mod tracker;
pub mod fusion;

pub use track::{FrameDetection, Track, TrackId, TrackUpdate};
pub use tracker::Tracker;
pub use fusion::{FusedTemplate, fuse_track};
