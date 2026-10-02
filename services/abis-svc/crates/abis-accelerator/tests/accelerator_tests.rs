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

//! Tests for accelerator interfaces and dispatcher.

use eel_abis_accelerator::cpu_fallback::CpuFallbackMatcher;
use eel_abis_accelerator::dispatcher::{select_accelerator, AcceleratorSelection};
use eel_abis_accelerator::domain::{AcceleratorKind, MatchQuery};
use eel_abis_accelerator::fpga::{FpgaMatcher, StubFpgaMatcher};
use eel_abis_accelerator::gpu::{GpuMatcher, StubGpuMatcher};
use uuid::Uuid;

fn query(n_gallery: usize, latency_ms: u64, energy: Option<u64>) -> MatchQuery {
    MatchQuery {
        probe_id: Uuid::now_v7(),
        probe_template: vec![128u8; 512],
        gallery_ids: (0..n_gallery).map(|_| Uuid::now_v7()).collect(),
        gallery_templates: (0..n_gallery).map(|_| vec![100u8; 512]).collect(),
        latency_budget_ms: latency_ms,
        energy_budget_mj: energy,
    }
}

#[test]
fn gpu_stub_returns_all_scores() {
    let matcher = StubGpuMatcher::new(AcceleratorKind::Cuda);
    let scores = matcher.batch_1n(&query(100, 200, None)).unwrap();
    assert_eq!(scores.len(), 100);
    assert!(scores.iter().all(|s| s.score == 0.5));
}

#[test]
fn gpu_stub_capabilities_healthy() {
    let matcher = StubGpuMatcher::new(AcceleratorKind::ROCm);
    let caps = matcher.capabilities();
    assert!(caps.healthy);
    assert_eq!(caps.kind, AcceleratorKind::ROCm);
}

#[test]
fn fpga_stub_returns_scores() {
    let matcher = StubFpgaMatcher::new("bs_v2");
    let scores = matcher.stream_match(&query(50, 100, Some(500))).unwrap();
    assert_eq!(scores.len(), 50);
    assert!(scores.iter().all(|s| s.score == 0.6));
}

#[test]
fn fpga_bitstream_id_returned() {
    assert_eq!(
        StubFpgaMatcher::new("bitstream_001").bitstream_id(),
        "bitstream_001"
    );
}

#[test]
fn cpu_fallback_matches_all_gallery() {
    let scores = CpuFallbackMatcher.batch_1n(&query(200, 500, None)).unwrap();
    assert_eq!(scores.len(), 200);
}

#[test]
fn cpu_fallback_score_in_range() {
    let scores = CpuFallbackMatcher.batch_1n(&query(10, 100, None)).unwrap();
    for s in &scores {
        assert!((0.0..=1.0).contains(&s.score));
    }
}

#[test]
fn dispatcher_selects_gpu_large_gallery() {
    assert_eq!(
        select_accelerator(&query(50_000, 200, None), true, true),
        AcceleratorSelection::Gpu
    );
}

#[test]
fn dispatcher_selects_fpga_medium_energy() {
    assert_eq!(
        select_accelerator(&query(5_000, 50, Some(100)), false, true),
        AcceleratorSelection::Fpga
    );
}

#[test]
fn dispatcher_selects_cpu_small_gallery() {
    assert_eq!(
        select_accelerator(&query(100, 50, None), true, true),
        AcceleratorSelection::Cpu
    );
}

#[test]
fn dispatcher_cpu_when_gpu_unhealthy() {
    assert_eq!(
        select_accelerator(&query(50_000, 200, None), false, false),
        AcceleratorSelection::Cpu
    );
}

#[test]
fn gpu_template_mismatch_errors() {
    let matcher = StubGpuMatcher::new(AcceleratorKind::Cuda);
    let q = MatchQuery {
        probe_id: Uuid::now_v7(),
        probe_template: vec![1u8; 512],
        gallery_ids: vec![Uuid::now_v7()],
        gallery_templates: vec![],
        latency_budget_ms: 100,
        energy_budget_mj: None,
    };
    assert!(matcher.batch_1n(&q).is_err());
}

#[test]
fn cpu_capabilities_kind_is_cpu() {
    assert_eq!(CpuFallbackMatcher.capabilities().kind, AcceleratorKind::Cpu);
}
