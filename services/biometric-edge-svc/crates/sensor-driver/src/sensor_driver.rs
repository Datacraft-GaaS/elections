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

/// Something a kit can acquire.
///
/// This is a *capture* taxonomy, deliberately wider than "biometric modality".
/// Two kinds of thing end up here:
///
/// - **Biometric samples**, which yield a template or image handled under the
///   biometric rules. Each cites the standard its data format comes from.
/// - **Document scans**, which yield machine-readable text and are *not*
///   biometric. [`Modality::is_biometric`] encodes that distinction so a
///   downstream handler cannot accidentally route an MRZ scan into biometric
///   template storage.
///
/// The matching side of these modalities lives in `abis-svc/crates/abis-modality-*`.
/// Capture and matching are separate concerns: a modality can be matchable with
/// no capture path (for example latent prints, which come from crime scenes and
/// are therefore deliberately absent here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    /// Contact fingerprint image set. ISO/IEC 19794-2.
    Fingerprint,
    /// Near-infrared finger vascular pattern. ISO/IEC 19794-9.
    FingerVein,
    /// Near-infrared palm vascular pattern. ISO/IEC 19794-9.
    PalmVein,
    /// Near-infrared wrist vascular pattern. ISO/IEC 19794-9.
    WristVein,
    /// Contact palm print. ISO/IEC 19794-2.
    PalmPrint,
    /// Face image. ISO/IEC 19794-5.
    Face,
    /// Face image with an active liveness challenge, ISO/IEC 30107-3 PAD.
    LivePhotograph,
    /// Periocular (eye-region) image, captured with the face. ISO/IEC 19794-5.
    Periocular,
    /// Iris image. ISO/IEC 19794-6.
    Iris,
    /// External ear image. See `abis-modality-ear` for the data format.
    Ear,
    /// Speaker voice recording. ISO/IEC 19794-13.
    Voiceprint,
    /// Gait image sequence or joint trajectory. ISO/IEC 39794-17.
    Gait,
    /// Handwritten signature. ISO/IEC 19794-7.
    Signature,
    /// Optical document scan with an ICAO 9303 machine-readable zone.
    ///
    /// Not a biometric. Kept in the same enum because it is acquired through
    /// the same capture path, and splitting it into a parallel enum would mean
    /// every call site handling "something a sensor produced" had to match two
    /// types.
    DocumentMrz,
}

impl Modality {
    /// Every modality, in declaration order. Used by exhaustive tests and by
    /// adapters that build a per-modality dispatch table.
    pub const ALL: &'static [Modality] = &[
        Modality::Fingerprint,
        Modality::FingerVein,
        Modality::PalmVein,
        Modality::WristVein,
        Modality::PalmPrint,
        Modality::Face,
        Modality::LivePhotograph,
        Modality::Periocular,
        Modality::Iris,
        Modality::Ear,
        Modality::Voiceprint,
        Modality::Gait,
        Modality::Signature,
        Modality::DocumentMrz,
    ];

    /// The data-format standard this modality's output conforms to, or `None`
    /// for modalities with no single binding standard.
    pub fn standard(self) -> Option<&'static str> {
        match self {
            Modality::Fingerprint | Modality::PalmPrint => Some("ISO/IEC 19794-2"),
            Modality::FingerVein | Modality::PalmVein | Modality::WristVein => {
                Some("ISO/IEC 19794-9")
            }
            Modality::Face | Modality::LivePhotograph | Modality::Periocular => {
                Some("ISO/IEC 19794-5")
            }
            Modality::Iris => Some("ISO/IEC 19794-6"),
            Modality::Voiceprint => Some("ISO/IEC 19794-13"),
            Modality::Gait => Some("ISO/IEC 39794-17"),
            Modality::Signature => Some("ISO/IEC 19794-7"),
            Modality::DocumentMrz => Some("ICAO 9303"),
            Modality::Ear => None,
        }
    }

    /// Returns `true` when the output is biometric data subject to the
    /// biometric handling rules, as opposed to document data.
    pub fn is_biometric(self) -> bool {
        !matches!(self, Modality::DocumentMrz)
    }

    /// Returns `true` when the subject must physically present the body part
    /// being captured, as opposed to being in frame or speaking.
    ///
    /// Contact and near-infrared vascular modalities need the subject's hand;
    /// gait and live photograph need the subject in view for a period. Only
    /// voice and document capture need no physical presentation.
    pub fn requires_body_presentation(self) -> bool {
        !matches!(self, Modality::Voiceprint | Modality::DocumentMrz)
    }

    /// The error returned when nothing is presented to the sensor.
    pub fn not_present_error(self) -> SensorError {
        match self {
            Modality::Fingerprint => SensorError::NoFingerPresent,
            Modality::FingerVein => SensorError::NoFingerVeinPresent,
            Modality::PalmVein => SensorError::NoPalmVeinPresent,
            Modality::WristVein => SensorError::NoWristVeinPresent,
            Modality::PalmPrint => SensorError::NoPalmPrintPresent,
            Modality::Face => SensorError::NoFacePresent,
            Modality::LivePhotograph => SensorError::NoFacePresent,
            Modality::Periocular => SensorError::NoFacePresent,
            Modality::Iris => SensorError::NoIrisPresent,
            Modality::Ear => SensorError::NoEarPresent,
            Modality::Voiceprint => SensorError::NoVoicePresent,
            Modality::Gait => SensorError::NoGaitPresent,
            Modality::Signature => SensorError::NoSignaturePresent,
            Modality::DocumentMrz => SensorError::NoDocumentPresent,
        }
    }
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
    #[error("no finger presented to the NIR vascular sensor")]
    NoFingerVeinPresent,
    #[error("no palm presented to the NIR vascular sensor")]
    NoPalmVeinPresent,
    #[error("no wrist presented to the NIR vascular sensor")]
    NoWristVeinPresent,
    #[error("no palm presented to the contact palm sensor")]
    NoPalmPrintPresent,
    #[error("no ear presented to the sensor")]
    NoEarPresent,
    #[error("no voice presented to the microphone")]
    NoVoicePresent,
    #[error("subject not in view for a full gait cycle")]
    NoGaitPresent,
    #[error("no signature presented to the pad")]
    NoSignaturePresent,
    #[error("no document presented to the reader")]
    NoDocumentPresent,
    #[error("presentation attack detected during live capture: {0}")]
    LivenessFailed(String),
    #[error("machine-readable zone unreadable after {attempts} attempts")]
    MrzUnreadable {
        /// How many read attempts were made before giving up.
        attempts: u32,
    },
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

/// Vendor-agnostic contract for capture on an edge kit.
///
/// # One required method
///
/// An adapter implements [`capture`](Self::capture) and nothing else. Every
/// per-modality method is a provided wrapper over it. This is deliberate: with
/// fourteen modalities, requiring one method each would mean a new vendor
/// adapter writes fourteen near-identical methods and any future modality
/// becomes a breaking change for every adapter in the fleet. One required
/// method keeps vendor selection a build-time choice, which is the point of
/// ADR-0026.
///
/// An adapter that genuinely cannot serve a modality should return
/// [`SensorError::UnsupportedModality`] from `capture` for that case rather
/// than panicking or returning a fabricated sample.
///
/// Implementations must be `Send + Sync` so the kit can hold a single sensor
/// behind an `Arc<dyn BiometricSensorDriver>` and drive it from concurrent
/// routes.
pub trait BiometricSensorDriver: Send + Sync {
    /// Acquire one sample of `modality` within `timeout`.
    ///
    /// This is the only method an adapter must implement.
    fn capture(
        &self,
        modality: Modality,
        timeout: Duration,
    ) -> Result<BiometricCapture, SensorError>;

    /// Capture a fingerprint within `timeout`.
    fn capture_fingerprint(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Fingerprint, timeout)
    }

    /// Capture a face within `timeout`.
    fn capture_face(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Face, timeout)
    }

    /// Capture an iris within `timeout`.
    fn capture_iris(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Iris, timeout)
    }

    /// Capture a near-infrared finger vascular pattern within `timeout`.
    fn capture_finger_vein(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::FingerVein, timeout)
    }

    /// Capture a near-infrared palm vascular pattern within `timeout`.
    fn capture_palm_vein(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::PalmVein, timeout)
    }

    /// Capture a near-infrared wrist vascular pattern within `timeout`.
    fn capture_wrist_vein(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::WristVein, timeout)
    }

    /// Capture a contact palm print within `timeout`.
    fn capture_palm_print(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::PalmPrint, timeout)
    }

    /// Capture a liveness-checked photograph within `timeout`.
    ///
    /// An adapter must not return a sample for this modality without an
    /// active presentation-attack check. A still photograph or replayed
    /// recording of the subject is a successful attack, so an adapter that
    /// cannot challenge the subject must fail rather than fall back to
    /// [`capture_face`](Self::capture_face).
    fn capture_live_photograph(
        &self,
        timeout: Duration,
    ) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::LivePhotograph, timeout)
    }

    /// Capture the periocular region within `timeout`.
    fn capture_periocular(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Periocular, timeout)
    }

    /// Capture an ear image within `timeout`.
    fn capture_ear(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Ear, timeout)
    }

    /// Capture a speaker voice recording within `timeout`.
    fn capture_voiceprint(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Voiceprint, timeout)
    }

    /// Capture a gait sequence within `timeout`.
    ///
    /// Gait needs the subject to walk a path, so the practical timeout is much
    /// longer than for a contact capture and the kit must give walking guidance.
    fn capture_gait(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Gait, timeout)
    }

    /// Capture a handwritten signature within `timeout`.
    fn capture_signature(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::Signature, timeout)
    }

    /// Scan a document's machine-readable zone within `timeout`.
    ///
    /// Returns document data, not a biometric. Adapters should retry a failed
    /// read internally and report [`SensorError::MrzUnreadable`] only once
    /// retries are exhausted, so the officer is not asked to re-present a
    /// document that merely needed a second angle.
    fn capture_document_mrz(&self, timeout: Duration) -> Result<BiometricCapture, SensorError> {
        self.capture(Modality::DocumentMrz, timeout)
    }

    /// Reports which of `supported` this adapter can actually acquire.
    ///
    /// Provided so a client can grey out an unavailable step in the UI rather
    /// than letting an officer start a capture that cannot finish. The default
    /// reports every modality as supported; adapters that serve a subset should
    /// override.
    fn supported_modalities(&self) -> Vec<Modality> {
        Modality::ALL.to_vec()
    }

    /// Returns `true` when this adapter can acquire `modality`.
    fn supports(&self, modality: Modality) -> bool {
        self.supported_modalities().contains(&modality)
    }

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
    /// Per-modality behaviour. A map rather than one field per modality so
    /// adding a modality to [`Modality`] does not require touching the mock,
    /// which is the same reason the trait has one required method.
    behaviour: std::collections::HashMap<Modality, MockAcquisition>,
    /// Modalities this mock claims to support; the rest report as unsupported.
    supported: Vec<Modality>,
}

impl MockSensorDriver {
    /// Build a mock that presents every modality with the given quality.
    pub fn present(device_serial: impl Into<String>, quality: u8) -> Self {
        Self {
            device_serial: device_serial.into(),
            quality,
            behaviour: Modality::ALL
                .iter()
                .map(|m| (*m, MockAcquisition::Present))
                .collect(),
            supported: Modality::ALL.to_vec(),
        }
    }

    /// Build a mock with configurable behaviour for the three original
    /// modalities and every other modality presented.
    pub fn with_behaviour(
        device_serial: impl Into<String>,
        quality: u8,
        fingerprint: MockAcquisition,
        face: MockAcquisition,
        iris: MockAcquisition,
    ) -> Self {
        Self::present(device_serial, quality)
            .with_modality(Modality::Fingerprint, fingerprint)
            .with_modality(Modality::Face, face)
            .with_modality(Modality::Iris, iris)
    }

    /// Set the behaviour for one modality.
    pub fn with_modality(mut self, modality: Modality, behaviour: MockAcquisition) -> Self {
        self.behaviour.insert(modality, behaviour);
        self
    }

    /// Restrict the mock to `modalities`, so the rest report as unsupported.
    ///
    /// Models a kit that physically has, say, a fingerprint reader and a camera
    /// but no vascular sensor, so a client can grey out the steps it cannot run.
    pub fn with_supported(mut self, modalities: &[Modality]) -> Self {
        self.supported = modalities.to_vec();
        self
    }

    fn capture_one(
        &self,
        modality: Modality,
        timeout: Duration,
    ) -> Result<BiometricCapture, SensorError> {
        if !self.supported.contains(&modality) {
            return Err(SensorError::UnsupportedModality);
        }
        let acquisition = self
            .behaviour
            .get(&modality)
            .copied()
            .unwrap_or(MockAcquisition::Present);
        match acquisition {
            // Raw acquisition returns the sample with its reported quality;
            // quality gating is the caller's policy (see quality_gate and the
            // degraded-finger fallback), so low-quality samples flow through.
            MockAcquisition::Present => Ok(self.build_capture(modality)),
            MockAcquisition::Absent => Err(modality.not_present_error()),
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
    fn capture(
        &self,
        modality: Modality,
        timeout: Duration,
    ) -> Result<BiometricCapture, SensorError> {
        tracing::debug!(device_serial = %self.device_serial, ?modality, "mock capture");
        self.capture_one(modality, timeout)
    }

    fn supported_modalities(&self) -> Vec<Modality> {
        self.supported.clone()
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

    // ---- extended modalities ------------------------------------------------

    /// Every modality must be capturable through the single required method.
    #[test]
    fn every_declared_modality_is_capturable() {
        let driver = MockSensorDriver::present("BVRD-KE-00412", 90);
        for modality in Modality::ALL {
            let capture = driver
                .capture(*modality, TIMEOUT)
                .unwrap_or_else(|e| panic!("{modality:?} should be capturable: {e}"));
            assert_eq!(capture.sample.meta.modality, *modality);
            assert!(!capture.sample.data.is_empty());
            assert!(!capture.degraded);
        }
        // The ALL list must not drift from the enum.
        assert_eq!(Modality::ALL.len(), 14);
    }

    /// The specific new capture methods the platform asked for.
    #[test]
    fn the_new_capture_methods_work() {
        let driver = MockSensorDriver::present("BVRD-KE-00412", 88);
        let cases: [(Modality, fn(&dyn BiometricSensorDriver, Duration) -> Result<BiometricCapture, SensorError>); 8] = [
            (Modality::PalmVein, |d, t| d.capture_palm_vein(t)),
            (Modality::FingerVein, |d, t| d.capture_finger_vein(t)),
            (Modality::WristVein, |d, t| d.capture_wrist_vein(t)),
            (Modality::LivePhotograph, |d, t| d.capture_live_photograph(t)),
            (Modality::DocumentMrz, |d, t| d.capture_document_mrz(t)),
            (Modality::Gait, |d, t| d.capture_gait(t)),
            (Modality::Voiceprint, |d, t| d.capture_voiceprint(t)),
            (Modality::PalmPrint, |d, t| d.capture_palm_print(t)),
        ];
        let boxed: &dyn BiometricSensorDriver = &driver;
        for (expected, call) in cases {
            let capture = call(boxed, TIMEOUT).unwrap_or_else(|e| panic!("{expected:?}: {e}"));
            assert_eq!(capture.sample.meta.modality, expected);
        }
    }

    #[test]
    fn each_modality_reports_its_absence_with_a_specific_error() {
        for modality in Modality::ALL {
            let driver = MockSensorDriver::present("kit", 90)
                .with_modality(*modality, MockAcquisition::Absent);
            let err = driver
                .capture(*modality, TIMEOUT)
                .expect_err(&format!("{modality:?} should fail when nothing is presented"));
            // The error must name the modality rather than being a generic miss.
            assert_eq!(err.to_string(), modality.not_present_error().to_string());
        }
    }

    #[test]
    fn document_mrz_is_not_a_biometric() {
        // An MRZ scan yields document text; routing it into biometric template
        // storage would misclassify it under the retention rules.
        assert!(!Modality::DocumentMrz.is_biometric());
        for modality in Modality::ALL.iter().filter(|m| **m != Modality::DocumentMrz) {
            assert!(modality.is_biometric(), "{modality:?} should be biometric");
        }
    }

    #[test]
    fn voice_and_document_do_not_need_body_presentation() {
        assert!(!Modality::Voiceprint.requires_body_presentation());
        assert!(!Modality::DocumentMrz.requires_body_presentation());
        assert!(Modality::Gait.requires_body_presentation());
        assert!(Modality::PalmVein.requires_body_presentation());
    }

    #[test]
    fn every_modality_cites_a_standard_or_explains_its_absence() {
        for modality in Modality::ALL {
            // Ear is the one documented exception; the rest must cite one.
            if *modality == Modality::Ear {
                assert!(modality.standard().is_none());
            } else {
                assert!(modality.standard().is_some(), "{modality:?} must cite a standard");
            }
        }
    }

    #[test]
    fn modalities_use_the_correct_standards() {
        // Guards the specific numbers, which are the easy thing to get wrong.
        assert_eq!(Modality::Fingerprint.standard(), Some("ISO/IEC 19794-2"));
        assert_eq!(Modality::PalmVein.standard(), Some("ISO/IEC 19794-9"));
        assert_eq!(Modality::Iris.standard(), Some("ISO/IEC 19794-6"));
        assert_eq!(Modality::Voiceprint.standard(), Some("ISO/IEC 19794-13"));
        // Gait moved to the 39794 series; it is NOT 19794-6, which is iris.
        assert_eq!(Modality::Gait.standard(), Some("ISO/IEC 39794-17"));
        assert_eq!(Modality::Signature.standard(), Some("ISO/IEC 19794-7"));
        assert_eq!(Modality::DocumentMrz.standard(), Some("ICAO 9303"));
    }

    #[test]
    fn an_unsupported_modality_is_reported_not_faked() {
        // A kit with no vascular sensor must say so, never return a sample.
        let driver = MockSensorDriver::present("kit", 90)
            .with_supported(&[Modality::Fingerprint, Modality::Face]);
        assert!(driver.supports(Modality::Fingerprint));
        assert!(!driver.supports(Modality::PalmVein));
        let err = driver.capture(Modality::PalmVein, TIMEOUT).unwrap_err();
        assert!(matches!(err, SensorError::UnsupportedModality));
    }

    #[test]
    fn modality_round_trips_through_json() {
        for modality in Modality::ALL {
            let text = serde_json::to_string(modality).unwrap();
            let decoded: Modality = serde_json::from_str(&text).unwrap();
            assert_eq!(&decoded, modality);
        }
        assert_eq!(
            serde_json::to_string(&Modality::DocumentMrz).unwrap(),
            "\"document_mrz\""
        );
    }

    #[test]
    fn mock_is_send_sync_usable_behind_arc() {
        let driver: std::sync::Arc<dyn BiometricSensorDriver> =
            std::sync::Arc::new(MockSensorDriver::present("BVRD-KE-00412", 91));
        let capture = driver.capture_iris(TIMEOUT).unwrap();
        assert_eq!(capture.sample.meta.modality, Modality::Iris);
    }
}
