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

pub use bfm::{Camera3d, DmmError, Fit3dmm, Model3dmm, Pose3d};
pub use spoof::{SpoofDecision, SpoofDetector};
