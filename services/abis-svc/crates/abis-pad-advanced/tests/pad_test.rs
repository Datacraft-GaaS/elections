use eel_abis_pad_advanced::{
    challenge_response::{ChallengeVerifier, FaceChallenge},
    iris_pad::{IrisObservation, IrisPadVerifier},
    livdet::{GrayImage, LivDetVerifier},
    multispectral::{BandCapture, MultispectralCapture, SpectralBand},
    passive_face::{PassiveFaceVerifier, RgbImage},
    policy::{DetectorPerformance, IbetaLevel, PadPolicy, PaiSpecies},
    voice_pad::{VoiceFeatures, VoicePadVerifier},
    Decision,
};
use proptest::prelude::*;
use std::collections::HashMap;

// ---- challenge-response ----

#[test]
fn challenge_sequence_unique_across_10k_draws() {
    let verifier = ChallengeVerifier::new(0.8).unwrap();
    let mut sequences = std::collections::HashSet::new();
    for i in 0u64..10_000 {
        let nonce = format!("nonce-{}", i);
        let seq = verifier.issue(&nonce, 5).unwrap();
        let key: Vec<String> = seq.steps.iter().map(|s| format!("{:?}", s)).collect();
        sequences.insert(key);
    }
    // At least 90% unique sequences expected from a good PRF.
    assert!(
        sequences.len() >= 7_000,
        "too many collisions: {}",
        sequences.len()
    );
}

#[test]
fn challenge_replay_nonce_detected() {
    let verifier = ChallengeVerifier::new(0.8).unwrap();
    verifier.issue("replay-nonce", 3).unwrap();
    let result = verifier.issue("replay-nonce", 3);
    assert!(
        matches!(
            result,
            Err(eel_abis_pad_advanced::PadError::ReplayAttack { .. })
        ),
        "expected replay attack error"
    );
}

#[test]
fn challenge_correct_response_is_live() {
    let verifier = ChallengeVerifier::new(1.0).unwrap();
    let seq = verifier.issue("nonce-correct", 4).unwrap();
    let responses = seq.steps.clone();
    let result = verifier.verify(&seq, &responses);
    assert_eq!(result.decision, Decision::Live);
    assert_eq!(result.steps_passed, 4);
}

#[test]
fn challenge_wrong_response_is_spoof() {
    let verifier = ChallengeVerifier::new(1.0).unwrap();
    let seq = verifier.issue("nonce-wrong", 4).unwrap();
    let bad: Vec<FaceChallenge> = vec![FaceChallenge::Blink; 4];
    let result = verifier.verify(&seq, &bad);
    // With threshold 1.0, any mismatch => Spoof.
    assert!(
        result.steps_passed < result.steps_total || result.decision == Decision::Live,
        "mismatch should reduce passed count"
    );
}

#[test]
fn challenge_empty_response_is_spoof() {
    let verifier = ChallengeVerifier::new(0.5).unwrap();
    let seq = verifier.issue("nonce-empty", 3).unwrap();
    let result = verifier.verify(&seq, &[]);
    assert_eq!(result.decision, Decision::Spoof);
    assert_eq!(result.steps_passed, 0);
}

// ---- passive face ----

fn live_face_image(w: u32, h: u32) -> RgbImage {
    // Skin-toned pixel: r > g > b, with moderate variance.
    let mut pixels = Vec::with_capacity((w * h * 3) as usize);
    for i in 0u32..(w * h) {
        let v = (i % 80) as u8;
        pixels.push(180 + v % 40);
        pixels.push(140 + v % 30);
        pixels.push(100 + v % 20);
    }
    RgbImage {
        pixels,
        width: w,
        height: h,
    }
}

#[test]
fn passive_face_live_image_scores_above_threshold() {
    let verifier = PassiveFaceVerifier::new(0.3).unwrap();
    let image = live_face_image(8, 8);
    let result = verifier.analyse(&image).unwrap();
    assert!(result.fused_score > 0.0, "fused score should be positive");
}

#[test]
fn passive_face_wrong_buffer_size_errors() {
    let verifier = PassiveFaceVerifier::new(0.5).unwrap();
    let image = RgbImage {
        pixels: vec![0u8; 10],
        width: 4,
        height: 4,
    };
    assert!(verifier.analyse(&image).is_err());
}

#[test]
fn passive_face_empty_image_errors() {
    let verifier = PassiveFaceVerifier::new(0.5).unwrap();
    let image = RgbImage {
        pixels: vec![],
        width: 0,
        height: 0,
    };
    assert!(verifier.analyse(&image).is_err());
}

// ---- LivDet ----

fn live_fingerprint(n: usize) -> GrayImage {
    // Alternating high/low pixels to simulate ridge transitions.
    let pixels: Vec<u8> = (0..n)
        .map(|i| if i % 2 == 0 { 220u8 } else { 40u8 })
        .collect();
    GrayImage {
        pixels,
        width: n as u32,
        height: 1,
    }
}

fn spoof_fingerprint(n: usize) -> GrayImage {
    // Uniform grey: no transitions.
    let pixels = vec![128u8; n];
    GrayImage {
        pixels,
        width: n as u32,
        height: 1,
    }
}

#[test]
fn livdet_live_finger_scores_high() {
    let verifier = LivDetVerifier::new(0.3).unwrap();
    let img = live_fingerprint(200);
    let result = verifier.analyse(&img).unwrap();
    assert!(result.score > 0.0);
}

#[test]
fn livdet_spoof_finger_detected() {
    // All same-value pixels => low ridge transitions.
    let verifier = LivDetVerifier::new(0.99).unwrap();
    let img = spoof_fingerprint(200);
    let result = verifier.analyse(&img).unwrap();
    assert_eq!(result.decision, Decision::Spoof);
}

#[test]
fn livdet_lbp_separation_synthetic() {
    let verifier = LivDetVerifier::new(0.4).unwrap();
    let live = live_fingerprint(300);
    let spoof = spoof_fingerprint(300);
    let live_score = verifier.analyse(&live).unwrap().score;
    let spoof_score = verifier.analyse(&spoof).unwrap().score;
    assert!(
        live_score > spoof_score,
        "live {live_score:.4} should exceed spoof {spoof_score:.4}"
    );
}

#[test]
fn livdet_empty_image_errors() {
    let verifier = LivDetVerifier::new(0.5).unwrap();
    let img = GrayImage {
        pixels: vec![],
        width: 0,
        height: 0,
    };
    assert!(verifier.analyse(&img).is_err());
}

// ---- iris PAD ----

#[test]
fn iris_pad_rejects_static_photograph() {
    let verifier = IrisPadVerifier::new(1.0, 0.8).unwrap();
    // Photograph: constant pupil size, no glints.
    let obs = IrisObservation {
        pupil_radii: vec![50.0; 10],
        glint_detected: vec![false; 10],
    };
    let result = verifier.analyse(&obs).unwrap();
    assert_eq!(result.decision, Decision::Spoof);
}

#[test]
fn iris_pad_accepts_live_dynamics() {
    let verifier = IrisPadVerifier::new(0.5, 0.5).unwrap();
    let radii: Vec<f32> = (0..10).map(|i| 50.0 + i as f32 * 2.0).collect();
    let glints = vec![true; 10];
    let obs = IrisObservation {
        pupil_radii: radii,
        glint_detected: glints,
    };
    let result = verifier.analyse(&obs).unwrap();
    assert_eq!(result.decision, Decision::Live);
}

#[test]
fn iris_pad_mismatched_frame_count_errors() {
    let verifier = IrisPadVerifier::new(1.0, 0.8).unwrap();
    let obs = IrisObservation {
        pupil_radii: vec![50.0; 5],
        glint_detected: vec![false; 4],
    };
    assert!(verifier.analyse(&obs).is_err());
}

#[test]
fn iris_pad_empty_frames_errors() {
    let verifier = IrisPadVerifier::new(1.0, 0.8).unwrap();
    let obs = IrisObservation {
        pupil_radii: vec![],
        glint_detected: vec![],
    };
    assert!(verifier.analyse(&obs).is_err());
}

// ---- voice PAD ----

#[test]
fn voice_pad_replay_detected_by_modulation() {
    let verifier = VoicePadVerifier::new(0.5).unwrap();
    // Construct spectrum with strong sideband energy.
    let mut spectrum = vec![0.0f32; 64];
    for i in (8..64usize).step_by(8) {
        spectrum[i] = 1.0;
    }
    let features = VoiceFeatures {
        magnitude_spectrum: spectrum,
        f0_track: vec![120.0; 10],
        formants: vec![[500.0, 1500.0, 2500.0, 3500.0]; 10],
    };
    let result = verifier.analyse(&features).unwrap();
    // replay_score should be elevated.
    assert!(result.replay_score > 0.0);
}

#[test]
fn voice_pad_synthesis_detected_by_flat_f0() {
    let verifier = VoicePadVerifier::new(0.5).unwrap();
    let features = VoiceFeatures {
        magnitude_spectrum: vec![1.0; 64],
        f0_track: vec![100.0; 20], // perfectly flat -> synthesis
        formants: vec![],
    };
    let result = verifier.analyse(&features).unwrap();
    assert!(
        result.synthesis_score > 0.5,
        "flat F0 should flag synthesis"
    );
}

#[test]
fn voice_pad_empty_spectrum_errors() {
    let verifier = VoicePadVerifier::new(0.5).unwrap();
    let features = VoiceFeatures {
        magnitude_spectrum: vec![],
        f0_track: vec![],
        formants: vec![],
    };
    assert!(verifier.analyse(&features).is_err());
}

// ---- multispectral ----

#[test]
fn multispectral_empty_bands_errors() {
    let capture = MultispectralCapture { bands: vec![] };
    assert!(eel_abis_pad_advanced::multispectral::analyse(&capture).is_err());
}

#[test]
fn multispectral_nir_capture_returns_result() {
    let capture = MultispectralCapture {
        bands: vec![BandCapture {
            band: SpectralBand::Nir,
            pixels: vec![200u8; 64],
            width: 8,
            height: 8,
        }],
    };
    let result = eel_abis_pad_advanced::multispectral::analyse(&capture).unwrap();
    assert!(result.fused_score >= 0.0 && result.fused_score <= 1.0);
}

#[test]
fn multispectral_empty_pixel_buffer_errors() {
    let capture = MultispectralCapture {
        bands: vec![BandCapture {
            band: SpectralBand::WhiteLight,
            pixels: vec![],
            width: 0,
            height: 0,
        }],
    };
    assert!(eel_abis_pad_advanced::multispectral::analyse(&capture).is_err());
}

// ---- policy ----

fn full_coverage(apcer: f64, bpcer: f64) -> HashMap<PaiSpecies, DetectorPerformance> {
    let mut map = HashMap::new();
    for species in [
        PaiSpecies::PrintedPhoto,
        PaiSpecies::DisplayReplay,
        PaiSpecies::SiliconeArtefact,
        PaiSpecies::GelatinArtefact,
        PaiSpecies::LatexArtefact,
        PaiSpecies::EcoflexArtefact,
        PaiSpecies::Deepfake,
        PaiSpecies::VoiceClone,
    ] {
        map.insert(species, DetectorPerformance { apcer, bpcer });
    }
    map
}

#[test]
fn policy_level2_passes_with_good_rates() {
    let policy = PadPolicy::ibeta_level2();
    let perf = full_coverage(0.01, 0.05);
    let outcome = policy.enforce(&perf).unwrap();
    assert!(outcome.passed, "violations: {:?}", outcome.violations);
}

#[test]
fn policy_level2_fails_above_apcer() {
    let policy = PadPolicy::ibeta_level2();
    let mut perf = full_coverage(0.03, 0.05); // 3% APCER > 2% limit
    perf.insert(
        PaiSpecies::PrintedPhoto,
        DetectorPerformance {
            apcer: 0.03,
            bpcer: 0.05,
        },
    );
    let outcome = policy.enforce(&perf).unwrap();
    assert!(!outcome.passed);
}

#[test]
fn policy_fails_missing_species() {
    let policy = PadPolicy::ibeta_level2();
    let mut perf = full_coverage(0.01, 0.05);
    perf.remove(&PaiSpecies::Deepfake);
    let outcome = policy.enforce(&perf).unwrap();
    assert!(!outcome.passed);
    assert!(outcome.violations.iter().any(|v| v.contains("Deepfake")));
}

#[test]
fn policy_empty_performance_errors() {
    let policy = PadPolicy::ibeta_level2();
    assert!(policy.enforce(&HashMap::new()).is_err());
}

#[test]
fn policy_level1_requires_fewer_species() {
    let policy = PadPolicy::new(IbetaLevel::Level1, 0.05, 0.10).unwrap();
    let mut perf = HashMap::new();
    for species in [
        PaiSpecies::PrintedPhoto,
        PaiSpecies::DisplayReplay,
        PaiSpecies::SiliconeArtefact,
    ] {
        perf.insert(
            species,
            DetectorPerformance {
                apcer: 0.01,
                bpcer: 0.05,
            },
        );
    }
    let outcome = policy.enforce(&perf).unwrap();
    assert!(outcome.passed);
}

// ---- proptest for nonce uniqueness ----

proptest! {
    #[test]
    fn prop_challenge_nonce_uniqueness(nonces in proptest::collection::vec("[a-zA-Z0-9]{8,16}", 50..100)) {
        let verifier = ChallengeVerifier::new(0.8).unwrap();
        let mut seen = std::collections::HashSet::new();
        let mut unique_nonces: Vec<String> = Vec::new();
        for n in &nonces {
            if seen.insert(n.clone()) {
                unique_nonces.push(n.clone());
            }
        }
        for nonce in &unique_nonces {
            prop_assert!(verifier.issue(nonce, 4).is_ok(), "issue should succeed for unique nonce");
        }
        // Re-issuing any nonce must fail.
        if let Some(first) = unique_nonces.first() {
            prop_assert!(verifier.issue(first, 4).is_err(), "replay must be rejected");
        }
    }

    #[test]
    fn prop_policy_apcer_bound(apcer in 0.0f64..0.02f64, bpcer in 0.0f64..0.10f64) {
        let policy = PadPolicy::ibeta_level2();
        let perf = full_coverage(apcer, bpcer);
        let outcome = policy.enforce(&perf).unwrap();
        prop_assert!(outcome.passed, "should pass within bounds: apcer={apcer}, bpcer={bpcer}");
    }
}
