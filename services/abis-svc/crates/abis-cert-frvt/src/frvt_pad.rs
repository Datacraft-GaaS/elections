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

//! FRVT PAD track submission.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrvtPadReport {
    pub apcer: f64,
    pub bpcer: f64,
    pub acer: f64,
    pub species_tested: Vec<String>,
}

pub struct FrvtPadSubmission {
    algorithm_id: String,
}

impl FrvtPadSubmission {
    pub fn new(algorithm_id: &str) -> Self {
        Self {
            algorithm_id: algorithm_id.to_owned(),
        }
    }

    /// Packages PAD performance into an FRVT PAD report card.
    pub fn run(&self, apcer: f64, bpcer: f64, species_tested: Vec<String>) -> FrvtPadReport {
        let acer = (apcer + bpcer) / 2.0;
        FrvtPadReport {
            apcer,
            bpcer,
            acer,
            species_tested,
        }
    }

    pub fn algorithm_id(&self) -> &str {
        &self.algorithm_id
    }
}
