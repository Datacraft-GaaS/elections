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

//! Match pipeline trait and wrappers.

use crate::domain::{MatchResult, TemplateIndex};

pub trait MatchPipeline: Send + Sync {
    fn match_1n(&self, probe_ref: &str, index: &[TemplateIndex], threshold: f32) -> MatchResult;
    fn match_11(&self, probe_ref: &str, gallery_ref: &str, threshold: f32) -> MatchResult;
}

/// Wrapper that holds a 1:N pipeline.
pub struct Pipeline1N {
    inner: Box<dyn MatchPipeline>,
}

impl Pipeline1N {
    pub fn new(inner: Box<dyn MatchPipeline>) -> Self {
        Self { inner }
    }

    pub fn run(&self, probe_ref: &str, index: &[TemplateIndex], threshold: f32) -> MatchResult {
        self.inner.match_1n(probe_ref, index, threshold)
    }
}

/// Wrapper that holds a 1:1 pipeline.
pub struct Pipeline11 {
    inner: Box<dyn MatchPipeline>,
}

impl Pipeline11 {
    pub fn new(inner: Box<dyn MatchPipeline>) -> Self {
        Self { inner }
    }

    pub fn run(&self, probe_ref: &str, gallery_ref: &str, threshold: f32) -> MatchResult {
        self.inner.match_11(probe_ref, gallery_ref, threshold)
    }
}
