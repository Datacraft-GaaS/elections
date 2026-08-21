# eel-abis-pad-advanced

iBeta PAD Level 2 presentation-attack detection for the eel ABIS stack.

## Coverage

| Modality | Module | Method |
|---|---|---|
| Fingerprint | `multispectral` | Multi-band (WL/NIR/SWIR/LWIR) liveness fusion |
| Fingerprint | `livdet` | LivDet-family ridge-frequency + valley-fill |
| Face | `challenge_response` | Blink/turn/smile/gaze with nonce-bound replay defence |
| Face | `passive_face` | LBP texture + colour moments + moire detection |
| Face | `deepfake` | VendorAdapter trait (MesoNet/FF++/RawNet2) |
| Iris | `iris_pad` | Pupil dynamics + Purkinje-reflex |
| Voice | `voice_pad` | Replay (modulation), TTS (F0 jitter), conversion (formants) |
| Policy | `policy` | iBeta Level 1/2 PAI coverage matrix enforcement |

## Policy

`PadPolicy::ibeta_level2()` enforces APCER <= 2% and BPCER <= 10% across all
required PAI species. Fail-closed: missing coverage or exceeded thresholds
produce a failed `PolicyOutcome`.

## Usage

```rust
use eel_abis_pad_advanced::policy::{PadPolicy, DetectorPerformance, PaiSpecies};
use std::collections::HashMap;

let policy = PadPolicy::ibeta_level2();
let mut perf = HashMap::new();
perf.insert(PaiSpecies::PrintedPhoto, DetectorPerformance { apcer: 0.01, bpcer: 0.05 });
// ... add remaining species ...
let outcome = policy.enforce(&perf).unwrap();
assert!(outcome.passed);
```

## Tests

Run with: `cargo test -p eel-abis-pad-advanced`

25+ unit tests plus proptest invariants covering nonce uniqueness and policy bounds.
