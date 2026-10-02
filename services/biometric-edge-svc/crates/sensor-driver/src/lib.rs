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

//! Vendor-agnostic biometric sensor driver abstraction (WP-3.8).
//!
//! The edge kit captures fingerprints, faces, and irises from hardware that
//! varies by contract (optical, capacitive, multispectral; RGB, NIR, 3D
//! structured-light; NIR iris). Rather than hard-coding one vendor SDK, the
//! platform binds capture behind the [`BiometricSensorDriver`] trait. Any
//! vendor hardware that exposes the required featureset plugs in as an
//! adapter; CI and local dev use the [`MockSensorDriver`] synthetic double.
#![warn(missing_docs)]
pub mod sensor_driver;

pub use sensor_driver::{
    BiometricCapture, BiometricSample, BiometricSensorDriver, CaptureMeta, CollectionMode,
    LivenessMethod, LivenessVerdict, MockAcquisition, MockSensorDriver, Modality, SensorError,
    QUALITY_GATE_MIN,
};
