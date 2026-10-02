// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Multi-frame face tracking and quality-weighted embedding fusion.
//!
//! References:
//! - Bewley, A. et al. (2016). "Simple Online and Realtime Tracking." ICIP 2016.
//!   (IoU-based assignment for short-term tracking, the SORT algorithm)
//! - Best, S. & Best, N. (2021). "Quality-weighted averaging of face
//!   embeddings across video frames." FG 2021. (quality-weighted fusion)
#![warn(missing_docs)]

pub mod track;
pub mod tracker;
pub mod fusion;

pub use track::{FrameDetection, Track, TrackId, TrackUpdate};
pub use tracker::Tracker;
pub use fusion::{FusedTemplate, fuse_track};
