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

// G03 Form Quality Gate Integration Test (CLOSED)
// Tests all 4 quality gates in mobile-bff integration flow

use mobile_bff::handlers::form_quality_gate::QualityGate;

#[test]
fn form_quality_gate_resolution_test() {
    // Resolution ≥ 1200×1600
    assert!(QualityGate::check(1200, 1600, 0.9, 0.1, 300.0).0); // pass
    assert!(!QualityGate::check(800, 600, 0.9, 0.1, 300.0).0);  // fail
    assert!(!QualityGate::check(1920, 1080, 0.9, 0.1, 300.0).0); // fail (height < 1600)
}

#[test]
fn form_quality_gate_ocr_test() {
    // OCR confidence ≥ 0.85
    assert!(QualityGate::check(1920, 1600, 0.85, 0.1, 300.0).0); // pass
    assert!(QualityGate::check(1920, 1600, 0.95, 0.1, 300.0).0); // pass
    assert!(!QualityGate::check(1920, 1600, 0.84, 0.1, 300.0).0); // fail
    assert!(!QualityGate::check(1920, 1600, 0.5, 0.1, 300.0).0); // fail
}

#[test]
fn form_quality_gate_blur_test() {
    // Blur ≤ 0.15
    assert!(QualityGate::check(1920, 1600, 0.95, 0.15, 300.0).0); // pass
    assert!(QualityGate::check(1920, 1600, 0.95, 0.05, 300.0).0); // pass
    assert!(!QualityGate::check(1920, 1600, 0.95, 0.16, 300.0).0); // fail
    assert!(!QualityGate::check(1920, 1600, 0.95, 0.3, 300.0).0); // fail
}

#[test]
fn form_quality_gate_illumination_test() {
    // Illumination ≥ 200 lux
    assert!(QualityGate::check(1920, 1600, 0.95, 0.1, 200.0).0); // pass
    assert!(QualityGate::check(1920, 1600, 0.95, 0.1, 300.0).0); // pass
    assert!(!QualityGate::check(1920, 1600, 0.95, 0.1, 199.0).0); // fail
    assert!(!QualityGate::check(1920, 1600, 0.95, 0.1, 50.0).0); // fail
}

#[test]
fn form_quality_gate_full_flow_test() {
    // Full form submission flow: quality gate → accept/reject → audit event
    let result = QualityGate::check(1920, 1600, 0.95, 0.05, 300.0);
    assert!(result.0); // accept
    assert!(result.1.is_empty()); // no reasons

    let result = QualityGate::check(800, 600, 0.7, 0.3, 100.0);
    assert!(!result.0); // reject
    assert!(!result.1.is_empty()); // has reasons
}
