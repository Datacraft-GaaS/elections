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
