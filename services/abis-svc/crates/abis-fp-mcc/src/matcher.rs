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

//! MCC end-to-end matcher.
//!
//! Reference: Cappelli, Ferrara, Maltoni, IEEE TPAMI 2010.

use abis_matcher::modality::fingerprint::FingerprintTemplate;

use crate::build::build_mcc;
use crate::consolidate::global_similarity;
use crate::params::MccParams;

/// MCC matcher: builds cylinders for both templates and runs LSSR consolidation.
#[derive(Default)]
pub struct MccMatcher {
    pub params: MccParams,
}

impl MccMatcher {
    pub fn new(params: MccParams) -> Self {
        Self { params }
    }

    /// Compare two fingerprint templates. Returns a score in [0.0, 1.0].
    pub fn compare(&self, probe: &FingerprintTemplate, gallery: &FingerprintTemplate) -> f32 {
        let probe_cyls = build_mcc(probe, &self.params);
        let gallery_cyls = build_mcc(gallery, &self.params);
        global_similarity(&probe_cyls, &gallery_cyls, self.params.min_matched)
    }
}
