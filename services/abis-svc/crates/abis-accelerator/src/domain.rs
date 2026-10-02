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

//! Domain types for accelerator interfaces.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub probe_id: Uuid,
    pub gallery_id: Uuid,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchQuery {
    pub probe_id: Uuid,
    pub probe_template: Vec<u8>,
    pub gallery_ids: Vec<Uuid>,
    pub gallery_templates: Vec<Vec<u8>>,
    pub latency_budget_ms: u64,
    pub energy_budget_mj: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceleratorKind {
    Cuda,
    ROCm,
    OneApi,
    Metal,
    Fpga,
    Cpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceleratorCapabilities {
    pub kind: AcceleratorKind,
    pub max_batch_size: usize,
    pub throughput_kps: f64,
    pub power_w: f64,
    pub healthy: bool,
}
