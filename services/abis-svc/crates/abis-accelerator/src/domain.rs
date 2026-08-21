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
