//! Accelerator dispatcher: selects GPU/FPGA/CPU based on query parameters.

use crate::domain::MatchQuery;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceleratorSelection {
    Gpu,
    Fpga,
    Cpu,
}

/// Select the optimal accelerator for a query.
/// GPU: gallery >= 10k AND latency budget >= 100ms.
/// FPGA: gallery >= 1k AND energy budget specified.
/// CPU: fallback.
pub fn select_accelerator(
    query: &MatchQuery,
    gpu_healthy: bool,
    fpga_healthy: bool,
) -> AcceleratorSelection {
    let n = query.gallery_ids.len();
    if gpu_healthy && n >= 10_000 && query.latency_budget_ms >= 100 {
        return AcceleratorSelection::Gpu;
    }
    if fpga_healthy && n >= 1_000 && query.energy_budget_mj.is_some() {
        return AcceleratorSelection::Fpga;
    }
    AcceleratorSelection::Cpu
}
