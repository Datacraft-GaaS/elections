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

//! eel-abis-cert-frvt: NIST FRVT participation infrastructure.
//!
//! Packages template extraction, verification pipeline, and report card
//! generation matching the FRVT 1:1, 1:N, Quality, and PAD tracks.
#![warn(missing_docs)]

pub mod frvt_1v1;
pub mod frvt_1vn;
pub mod frvt_pad;
pub mod frvt_quality;

pub use frvt_1v1::{Frvt1v1Error, Frvt1v1Report, Frvt1v1Submission};
pub use frvt_1vn::{Frvt1vNReport, Frvt1vNSubmission};
pub use frvt_pad::{FrvtPadReport, FrvtPadSubmission};
pub use frvt_quality::{FrvtQualityReport, FrvtQualitySubmission};
