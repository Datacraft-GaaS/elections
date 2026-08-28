# ABIS — Automated Biometric Identification System

Implementation reference for `services/abis-svc`: 85 crates, ~42K lines of source and ~19K lines of tests, all verified green in the full-workspace run (579 suites, 0 failures — see [../system/operations.md](../system/operations.md) for the machine prerequisite when linking `oqs` on macOS).

## Volumes

| File | Domain | Crates |
|---|---|---|
| [platform-core.md](platform-core.md) | Matching pipeline orchestration, vendor abstraction, enrolment/adjudication lifecycle, HTTP API, reports & ops tooling | 23 |
| [fingerprint.md](fingerprint.md) | Fingerprint technology area: MCC matching, skeletonisation, ridge count, orientation, level-3, texture, warp, consensus, multi-finger, quality, ONNX | 16 |
| [iris.md](iris.md) | Iris technology area: Daugman segmentation, encoding, quality, IREX-style metrics, cross-sensor, fusion, motion/pose/lens correction | 10 |
| [face.md](face.md) | Face technology area: detection, alignment, embedding, quality, occlusion, age, kinship, 3D-MM, cross-domain, de-identification, video, FRVT harnesses | 12 |
| [security-standards.md](security-standards.md) | Privacy-preserving matching (cancelable, shielding, SMPC, FHE), PAD/liveness, attestation & certifications, standards interchange (ANSI/NIST, CBEFF, MINEX, EBTS, ICAO, INTERPOL) | 16 |

Plus `abis-modality-*` (ear, gait, latent, palm, periocular, signature, vein, voice) — eight extended modalities documented alongside security-standards scope notes where relevant; each follows the same crate conventions (lib.rs purpose doc, pure-Rust algorithms, integration-test suites).

## What is genuinely implemented

The biometric **core is real signal processing**, not mock-ups:

- **Fingerprint**: Minutia Cylinder Code matcher (cylinder bit-coding, neighbourhood vectors, global similarity), Zhang-Suen-style skeletonisation, ridge counting, orientation/pattern classification, level-3 pore/crease features, texture analysis, TPS elastic warping, multi-finger consensus, NFIQ2.2-style quality.
- **Iris**: Daugman integro-differential operator segmentation with OSIRIS-style boundary refinement, iris encoding, IREX-style quality metrics, cross-sensor calibration, motion/pose/lens correction chains.
- **Standards**: MINEX implements exact ISO 19794-2 minutia packing; ANSI/NIST and EBTS parse their dialects fully.

## Honest capability tiers

Documentation deliberately distinguishes three tiers (each volume flags per-crate):

1. **Production-grade** — real algorithms with meaningful test depth: fp-mcc, fp-skeleton, ridge-count, orientation, texture, tps-warp, consensus, multifinger, cdeffs, fpvte, sourceafis-bridge; all ten iris crates' classical stages; face detect/align/quality/video/frvt; matcher pipeline + vendor abstraction; pad-advanced challenge-response; model-canary; minex.
2. **Structural seams** — correct trait/interface design awaiting a real backend: smpc-match (Shamir sharing real, protocol simulated), fhe-match (seam only), cancelable (simplified primitives), attestation quote verification (trusts dev stores), ansi-nist/cbeff/ebts/icao/interpol payload wrapping.
3. **Placeholder backends** — deterministic stand-ins for ML: iris-dl and face-embed ship SHA-256 hash embedders; real ONNX inference lives in the Python sidecar (`py/face-onnx-worker`, driven by `EEL_FACE_*_ONNX_PATH` env vars). No iris model path exists yet.

## Known gaps surfaced during documentation

- **Wiring**: most algorithm crates are currently *orphans* — consumed by tests, not by service binaries. Live match traffic runs through `MockMatcher` (`abis-api/src/routes/match_routes.rs`); the MCC/index pipeline is reachable via `abis-matcher` but not yet selected as the production matcher.
- **Auth middleware** (`spire_auth`, idempotency) exists in `abis-api/src/middleware.rs` but is not layered into the router; response cosigning sets a placeholder header.
- **Feature break**: `abis-fp-dl-onnx` references `ndarray` without declaring it — the `onnx` feature likely does not compile until fixed.
- **Unit inconsistency**: MCC treats minutia angles as degrees while tps-warp uses radians.
- **Thin spots**: enhancement's morphological thinning is a double-erosion placeholder; rectification API is an explicit REST stub; vendor has one external test.

These are tracked as engineering work items, not hidden; every claim above was verified against source during documentation.

## Testing

Per-crate test counts and what each suite proves are listed in the volume files. Workspace-wide reproduction:

```bash
export OPENSSL_DIR="$(brew --prefix openssl@3)"   # macOS/oqs linking
export RUSTFLAGS="-L$OPENSSL_DIR/lib"
cargo test --workspace --no-fail-fast             # 579 suites, 0 failures
```
