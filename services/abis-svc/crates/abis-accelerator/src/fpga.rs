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

//! FPGA matcher trait and stub implementation.

use crate::domain::{AcceleratorCapabilities, AcceleratorKind, MatchQuery, Score};
use crate::error::AcceleratorError;

pub trait FpgaMatcher: Send + Sync {
    fn capabilities(&self) -> AcceleratorCapabilities;
    fn stream_match(&self, query: &MatchQuery) -> Result<Vec<Score>, AcceleratorError>;
    fn health(&self) -> bool;
    fn bitstream_id(&self) -> &str;
}

pub struct StubFpgaMatcher {
    pub bitstream: String,
}

impl StubFpgaMatcher {
    pub fn new(bitstream: impl Into<String>) -> Self {
        Self {
            bitstream: bitstream.into(),
        }
    }
}

impl FpgaMatcher for StubFpgaMatcher {
    fn capabilities(&self) -> AcceleratorCapabilities {
        AcceleratorCapabilities {
            kind: AcceleratorKind::Fpga,
            max_batch_size: 50_000,
            throughput_kps: 1000.0,
            power_w: 75.0,
            healthy: true,
        }
    }

    fn stream_match(&self, query: &MatchQuery) -> Result<Vec<Score>, AcceleratorError> {
        if !self.health() {
            return Err(AcceleratorError::Unhealthy);
        }
        Ok(query
            .gallery_ids
            .iter()
            .map(|&gid| Score {
                probe_id: query.probe_id,
                gallery_id: gid,
                score: 0.6,
            })
            .collect())
    }

    fn health(&self) -> bool {
        true
    }

    fn bitstream_id(&self) -> &str {
        &self.bitstream
    }
}
