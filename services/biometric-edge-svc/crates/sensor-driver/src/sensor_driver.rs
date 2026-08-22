//! `BiometricSensorDriver` trait, its error type, and the `MockSensorDriver`
//! synthetic double used in CI.
//!
//! The trait is the vendor-agnostic contract for biometric capture on the edge
//! kit. A vendor adapter implements the three capture methods by translating
//! its SDK or protocol into the trait's result shapes; the provided
//! `quality_gate` and `capture_fingerprint_degraded` helpers implement the
//! shared quality policy so every adapter behaves identically.
use std::time::Duration;

use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

/// Minimum acceptable capture quality (0-100) for a biometric sample.
pub const QUALITY_GATE_MIN: u8 = 70;

/// Biometric modality being captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Fingerprint,
    Face,
    Iris,
}

/// Errors produced during a capture or quality evaluation.
#[derive(Debug, Error)]
pub enum SensorError {
    #[error("no finger presented to the sensor")]
    NoFingerPresent,
    #[error("no face presented to the sensor")]
    NoFacePresent,
    #[error("no iris presented to the sensor")]
    NoIrisPresent,
    #[error("capture timed out after {0:?}")]
    Timeout(Duration),
    #[error("captured quality {got} below gate minimum {min}")]
    QualityBelowGate { got: u8, min: u8 },
    #[error("sensor hardware failure: {0}")]
    Hardware(String),
    #[error("unsupported modality")]
    UnsupportedModality,
}

/// Metadata describing how a sample was captured.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureMeta {
    /// Modality that produced this sample.
    pub modality: Modality,
    /// Device serial or identifier of the kit sensor.
    pub device_serial: String,
    /// Unique sample id used for content-addressable storage.
    pub sample_id: String,
    /// Unix milliseconds at capture time.
    pub captured_at_ms: i64,
    /// Quality score 0-100 as reported by the sensor's quality engine.
    pub quality: u8,
}

/// A raw biometric capture plus its metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiometricSample {
    /// Raw sensor bytes (image or ISO/IEC 19794 encoded template).
    pub data: Vec<u8>,
    /// Capture metadata.
    pub meta: CaptureMeta,
}

/// A completed capture, with a degraded flag when the sample fell back to a
/// lower-quality gate rather than being rejected outright.
#[derive(Debug, Clone)]
pub struct BiometricCapture {
    /// The captured sample.
    pub sample: BiometricSample,
    /// True when the sample passed a relaxed gate after a degraded-finger
    /// fallback, informing downstream quality decisions.
    pub degraded: bool,
}

/// Vendor-agnostic contract for biometric capture on an edge kit.
///
/// Implementations must be `Send + Sync` so the kit can hold a single sensor
/// behind an `Arc<dyn BiometricSensorDriver>` and drive it from concurrent
/// routes.
pub trait BiometricSensorDriver: Send + Sync {
    /// Capture a fingerprint within `timeout`.
    fn capture_fingerprint(&self, timeout: Duration) -> Result<BiometricCapture, SensorError>;

    /// Capture a face within `timeout`.
    fn capture_face(&self, timeout: Duration) -> Result<BiometricCapture, SensorError>;

    /// Capture an iris within `timeout`.
    fn capture_iris(&self, timeout: Duration) -> Result<BiometricCapture, SensorError>;

    /// Validate a quality score against the shared gate minimum.
    ///
    /// Provided so all adapters enforce the same quality policy. Returns the
    /// score unchanged when it meets the gate.
    fn quality_gate(&self, quality: u8) -> Result<u8, SensorError> {
        if quality < QUALITY_GATE_MIN {
            return Err(SensorError::QualityBelowGate {
                got: quality,
                min: QUALITY_GATE_MIN,
            });
        }
        Ok(quality)
    }

    /// Degraded-finger fallback: re-acquire when the first pass fails the
    /// quality gate, then return the best sample marked as degraded.
    ///
    /// The relaxed gate (`QUALITY_GATE_MIN / 2`) is only applied on retry and
    /// the returned capture carries `degraded: true` so the backend can treat
    /// it as lower-confidence. A hardware absence or timeout on the retry is
    /// propagated unchanged.
    fn capture_fingerprint_degraded(
        &self,
        timeout: Duration,
    ) -> Result<BiometricCapture, SensorError> {
        // The strict gate is a policy check on the raw capture; a low-quality
        // sample passes through so the fallback has something to act on.
        let first = self.capture_fingerprint(timeout)?;
        if first.sample.meta.quality >= QUALITY_GATE_MIN {
            return Ok(first);
        }
        let retry = self.capture_fingerprint(timeout)?;
        if retry.sample.meta.quality < QUALITY_GATE_MIN / 2 {
            return Err(SensorError::QualityBelowGate {
                got: retry.sample.meta.quality,
                min: QUALITY_GATE_MIN / 2,
            });
        }
        Ok(BiometricCapture {
            sample: retry.sample,
            degraded: true,
        })
    }
}

/// Per-modality behaviour a mock sensor reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockAcquisition {
    /// A sample is presented and captured.
    Present,
    /// No subject is presented to the sensor.
    Absent,
    /// Capture times out.
    Timeout,
}

/// Synthetic double implementing [`BiometricSensorDriver`] for CI and local
/// dev. Behaviour is fixed per modality at construction so tests are
/// deterministic and never require physical hardware.
pub struct MockSensorDriver {
    device_serial: String,
    quality: u8,
    fingerprint: MockAcquisition,
    face: MockAcquisition,
    iris: MockAcquisition,
}

impl MockSensorDriver {
    /// Build a mock that presents all modalities with the given quality.
    pub fn present(device_serial: impl Into<String>, quality: u8) -> Self {
        Self {
            device_serial: device_serial.into(),
            quality,
            fingerprint: MockAcquisition::Present,
            face: MockAcquisition::Present,
            iris: MockAcquisition::Present,
        }
    }

    /// Build a mock with fully configurable per-modality behaviour.
    pub fn with_behaviour(
        device_serial: impl Into<String>,
        quality: u8,
        fingerprint: MockAcquisition,
        face: MockAcquisition,
        iris: MockAcquisition,
    ) -> Self {
        Self {
            device_serial: device_serial.into(),
            quality,
            fingerprint,
            face,
            iris,
        }
    }

    fn capture(
        &self,
        modality: Modality,
        acquisition: MockAcquisition,
        timeout: Duration,
    ) -> Result<BiometricCapture, SensorError> {
        match acquisition {
            // Raw acquisition returns the sample with its reported quality;
            // quality gating is the caller's policy (see quality_gate and the
            // degraded-finger fallback), so low-quality samples flow through.
            MockAcquisition::Present => Ok(self.build_capture(modality)),
            MockAcquisition::Absent => Err(match modality {
                Modality::Fingerprint => SensorError::NoFingerPresent,
                Modality::Face => SensorError::NoFacePresent,
                Modality::Iris => SensorError::NoIrisPresent,
            }),
            MockAcquisition::Timeout => Err(SensorError::Timeout(timeout)),
        }
    }

    fn build_capture(&self, modality: Modality) -> BiometricCapture {
        // Deterministic placeholder bytes derived from modality and quality so
        // tests can assert non-empty, content-addressed samples.
        let mut data = vec![0u8; 32];
        data[0] = modality as u8;
        data[1] = self.quality;
        BiometricCapture {
            sample: BiometricSample {
                data,
                meta: CaptureMeta {
                    modality,
                    device_serial: self.device_serial.clone(),
                    sample_id: Uuid::new_v4().to_string(),
                    captured_at_ms: Utc::now().timestamp_millis(),
                    quality: self.quality,
                },
            },
            degraded: false,
        }
    }
}

impl BiometricSensorDriver for MockSensorDriver {
    fn capture_fingerprint(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        tracing::debug!(device_serial = %self.device_serial, "mock fingerprint capture");
        self.capture(Modality::Fingerprint, self.fingerprint, timeout)
    }

    fn capture_face(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        tracing::debug!(device_serial = %self.device_serial, "mock face capture");
        self.capture(Modality::Face, self.face, timeout)
    }

    fn capture_iris(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        tracing::debug!(device_serial = %self.device_serial, "mock iris capture");
        self.capture(Modality::Iris, self.iris, timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIMEOUT: Duration = Duration::from_secs(5);

    #[test]
    fn present_fingerprint_meets_gate() {
        let driver = MockSensorDriver::present("BVRD-KE-00412", 88);
        let capture = driver.capture_fingerprint(TIMEOUT).unwrap();
        assert_eq!(capture.sample.meta.modality, Modality::Fingerprint);
        assert!(capture.sample.meta.quality >= QUALITY_GATE_MIN);
        assert!(!capture.degraded);
        assert!(!capture.sample.data.is_empty());
    }

    #[test]
    fn below_gate_quality_rejected() {
        let driver = MockSensorDriver::present("BVRD-KE-00412", 45);
        let err = driver.quality_gate(45).unwrap_err();
        assert!(matches!(
            err,
            SensorError::QualityBelowGate {
                got: 45,
                min: QUALITY_GATE_MIN
            }
        ));
        // Raw capture still flows through so the fallback can act on it.
        let capture = driver.capture_fingerprint(TIMEOUT).unwrap();
        assert_eq!(capture.sample.meta.quality, 45);
    }

    #[test]
    fn absent_finger_returns_no_finger() {
        let driver = MockSensorDriver::with_behaviour(
            "BVRD-KE-00412",
            90,
            MockAcquisition::Absent,
            MockAcquisition::Present,
            MockAcquisition::Present,
        );
        let err = driver.capture_fingerprint(TIMEOUT).unwrap_err();
        assert!(matches!(err, SensorError::NoFingerPresent));
    }

    #[test]
    fn timeout_propagated() {
        let driver = MockSensorDriver::with_behaviour(
            "BVRD-KE-00412",
            90,
            MockAcquisition::Timeout,
            MockAcquisition::Present,
            MockAcquisition::Present,
        );
        let err = driver.capture_fingerprint(TIMEOUT).unwrap_err();
        assert!(matches!(err, SensorError::Timeout(t) if t == TIMEOUT));
    }

    #[test]
    fn degraded_finger_fallback_lowers_gate() {
        // First pass quality 50 fails the strict gate (70); retry returns the
        // same quality 50 which passes the relaxed gate (35) and is degraded.
        let driver = MockSensorDriver::present("BVRD-KE-00412", 50);
        let capture = driver.capture_fingerprint_degraded(TIMEOUT).unwrap();
        assert!(capture.degraded);
        assert_eq!(capture.sample.meta.quality, 50);
    }

    #[test]
    fn degraded_finger_fallback_rejects_very_low_quality() {
        // Quality 20 fails even the relaxed gate (35) after retry.
        let driver = MockSensorDriver::present("BVRD-KE-00412", 20);
        let err = driver.capture_fingerprint_degraded(TIMEOUT).unwrap_err();
        assert!(matches!(err, SensorError::QualityBelowGate { .. }));
    }

    #[test]
    fn degraded_finger_fallback_passes_clean_first_capture() {
        let driver = MockSensorDriver::present("BVRD-KE-00412", 92);
        let capture = driver.capture_fingerprint_degraded(TIMEOUT).unwrap();
        assert!(!capture.degraded);
        assert!(capture.sample.meta.quality >= QUALITY_GATE_MIN);
    }

    #[test]
    fn mock_is_send_sync_usable_behind_arc() {
        let driver: std::sync::Arc<dyn BiometricSensorDriver> =
            std::sync::Arc::new(MockSensorDriver::present("BVRD-KE-00412", 91));
        let capture = driver.capture_iris(TIMEOUT).unwrap();
        assert_eq!(capture.sample.meta.modality, Modality::Iris);
    }
}
