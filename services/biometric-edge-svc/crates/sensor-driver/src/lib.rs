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
