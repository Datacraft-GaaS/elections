# Iris Crates (`abis-iris-*`)

Part of the ABIS (Automated Biometric Identification System) service under
`services/abis-svc/crates/`. Each directory is a Cargo package named
`eel-abis-<dir>` (e.g. `abis-iris-segment` = `eel-abis-iris-segment`).

## Package map and source size

| Directory | Package | src lines | Inline test modules | Inline `#[test]` fns |
|---|---|---:|---:|---:|
| abis-iris-segment | eel-abis-iris-segment | 942 | 4 | 19 |
| abis-iris-encode | eel-abis-iris-encode | 825 | 5 | 23 |
| abis-iris-quality | eel-abis-iris-quality | 704 | 3 | 23 |
| abis-iris-irex | eel-abis-iris-irex | 784 | 4 | 21 |
| abis-iris-crosssensor | eel-abis-iris-crosssensor | 348 | 2 | 15 |
| abis-iris-fusion | eel-abis-iris-fusion | 443 | 2 | 21 |
| abis-iris-motion | eel-abis-iris-motion | 493 | 3 | 22 |
| abis-iris-pose | eel-abis-iris-pose | 358 | 2 | 16 |
| abis-iris-lens | eel-abis-iris-lens | 640 | 4 | 24 |
| abis-iris-dl | eel-abis-iris-dl | 329 | 2 | 16 |

Line counts total each crate's `src/*.rs`. None of the ten crates has a
`tests/` integration-test directory; all verification is inline `#[cfg(test)]`
modules in `src`.

## Interconnections

From the crates' `Cargo.toml` files:

```
eel-abis-iris-segment   (no workspace deps; base crate)
eel-abis-iris-encode    -> iris-segment
eel-abis-iris-quality   -> iris-segment
eel-abis-iris-motion    -> iris-segment
eel-abis-iris-lens      -> iris-segment
eel-abis-iris-dl        -> iris-segment (+ sha2, hex)
eel-abis-iris-crosssensor -> iris-encode
eel-abis-iris-fusion    -> iris-encode
eel-abis-iris-pose      -> iris-segment, iris-encode
eel-abis-iris-irex      -> iris-encode, iris-quality, iris-dl, iris-fusion
```

`iris-irex` is therefore the top of the dependency chain. All crates share
`serde`, `thiserror`, `tracing` from the workspace; dev-dependencies are `rstest`
(and `serde_json` where serde round-trips are tested).

## abis-iris-segment

**Purpose** (lib.rs doc comment): "Iris segmentation: Daugman integro-
differential operator + OSIRIS boundary refinement." Cites Daugman (1993) and
Othman/Dorizzi/Garcia-Salicetti OSIRIS (2016).

**Algorithms**
- Daugman integro-differential operator: coarse centre/radius search maximising
  the finite-difference derivative of Gaussian-smoothed annular intensity
  averages — `src/daugman.rs:44 locate_circle`, with
  `src/daugman.rs:96 compute_annular_averages` and
  `src/daugman.rs:122 gaussian_smooth_1d`. Parameters in
  `DaugmanParams` (`min_radius=15`, `max_radius=120`, 64 angular samples,
  sigma 2.0, centre step 4).
- OSIRIS-style Viterbi boundary refinement on radial edge profiles; per-angle
  best radius selected by gradient magnitude, refined radius is the mean —
  `src/osiris.rs:16 viterbi_refine`.
- Occlusion mask construction including eyelid parabola fitting —
  `src/mask.rs:15 build_iris_mask`, `src/mask.rs:72 fit_eyelid_parabolas`.
- Pipeline orchestration plus an iris/sclera ratio sanity check —
  `src/segment.rs:96 SegmentIris::segment`,
  `src/segment.rs:146 compute_iris_sclera_ratio`.
- Shared types: `GrayImage`, `Circle`, `IrisMask` (with `valid_fraction`,
  `and`) defined in `lib.rs`.

**Public API**

| Item | Kind |
|---|---|
| `SegmentIris` / `IrisSegmentation` / `SegmentConfig` / `SegmentError` | struct / re-exported from `segment` |
| `locate_circle`, `DaugmanParams` | fn, struct (`daugman`) |
| `viterbi_refine` | fn (`osiris`) |
| `build_iris_mask` | fn (`mask`) |
| `GrayImage`, `Circle`, `IrisMask` | types (`lib.rs`) |

**Verification**: no `tests/` directory. 4 inline `#[cfg(test)]` modules, 19
`#[test]` fns (daugman 3, mask 5, osiris 3, segment 8): pupil localisation
within 8 px on synthetic irises; specular/eyelash/eyelid masking; Viterbi
stays near the true radius without moving the centre; segmentation
success/error paths; sclera ratio bounds; usable-fraction/mask consistency.

## abis-iris-encode

**Purpose**: "Iris template encoding: Daugman rubber-sheet normalisation + 2D
Gabor IrisCode." Also defines PIE ("Prickle Interpolated Encoding"), an internal
rotation-robust variant. Constants: normalised image is 512x64, IrisCode is
2048 bits.

**Algorithms**
- Daugman rubber-sheet unwrap to 512x64 with linear pupil-to-limbus
  interpolation between boundary points — `src/normalise.rs:31 rubber_sheet`,
  `src/normalise.rs:56 interpolate_point`.
- Complex Gabor filter bank (real part evaluated over a windowed kernel):
  `src/gabor.rs:32 GaborFilter::respond`; standard bank is 8 frequencies x 4
  orientations = 32 filters — `src/gabor.rs:62 standard_filter_bank`.
- Daugman IrisCode: bit = sign of paired Gabor responses with noise mask;
  `src/daugman_code.rs:63 encode_daugman`; masked Hamming distance requiring a
  minimum valid-bit overlap — `src/daugman_code.rs:31 DaugmanCode::hamming_distance`.
- PIE rotation-robust code: circular shifts of +/-3 columns (~2 degrees),
  majority vote over sign bits and validity — `src/pie.rs:36 encode_pie`,
  shift helpers at `pie.rs:88` and `pie.rs:103`.
- Facade encoder tying normalisation + coding together:
  `src/encode.rs:76 IrisEncoder::encode`; template distance
  `src/encode.rs:28 IrisTemplate::hamming_distance`.
- Fixed-size byte-array serde helpers in `src/serde_array.rs`.

**Public API**

| Item | Kind |
|---|---|
| `IrisEncoder`, `IrisTemplate`, `EncodeError` | struct/trait surface re-exported from `encode` |
| `rubber_sheet`, `NormalisedIris` | fn, struct (`normalise`) |
| `GaborFilter`, `standard_filter_bank` | struct, fn (`gabor`) |
| `encode_daugman`, `DaugmanCode` | fn, struct (`daugman_code`) |
| `encode_pie`, `PieCode` | fn, struct (`pie`) |
| `NORM_WIDTH/HEIGHT`, `IRIS_CODE_BITS/BYTES` | consts (`lib.rs`) |

**Verification**: no `tests/` directory. 5 inline modules, 23 tests: normalised
output exactly 512x64, deterministic, interpolation endpoints on pupil/limbus;
32-filter bank with finite responses; codes are 2048 bits; identical inputs ->
identical codes; self HD = 0; distances in range; mask mismatch errors; PIE
shift wrapping.

## abis-iris-quality

**Purpose**: "Iris image quality assessment per ISO/IEC 29794-6" (ISO/IEC
29794-6:2021).

**Algorithms**
- Annulus pixel statistics (inside limbus, outside pupil, mask-valid):
  `src/metrics.rs:98 annulus_pixels`.
- Sharpness via gradient energy over the annulus — `src/metrics.rs:16 sharpness`;
  contrast via p10/p90 spread — `src/metrics.rs:57 contrast`.
- Pupil/limbus dilation ratio (ideal 0.3-0.5) — `src/metrics.rs:70 dilation`;
  gaze angle via asin of normalised pupil offset — `src/metrics.rs:79 gaze_angle`;
  occlusion percentage — `src/metrics.rs:93 occlusion_pct`.
- Score mappings: piecewise dilation score `src/quality.rs:80
  dilation_quality_score`; linear gaze score (1.0 at 0 deg, 0.0 at 15 deg)
  `src/quality.rs:92 gaze_quality_score`; aggregate report
  `src/quality.rs:38 QualityReport::compute`.
- Accept/reject policy over the report with configurable bounds —
  `src/policy.rs:105 QualityPolicy::evaluate`, threshold validation
  `policy.rs:83 validate`.

**Public API**

| Item | Kind |
|---|---|
| `QualityReport`, `QualityError` | struct, enum (`quality`) |
| `QualityPolicy`, `PolicyError`, `Verdict`, `RejectionReasons` | struct/enums (`policy`) |
| `sharpness`, `contrast`, `dilation`, `gaze_angle`, `occlusion_pct` | fns (`metrics`) |

**Verification**: no `tests/` directory. 3 inline modules, 23 tests: metrics in
range on synthetic irises; tiny-image errors; dilation/gaze score anchors (1.0
ideal/frontal, 0.0 at 15 deg); occlusion 0/1 for full/zero masks; sharper images
score higher; policy accepts good samples and rejects each failure mode;
default policy validates.

## abis-iris-irex

**Purpose**: "NIST IREX (Iris Exchange) evaluation framework": corpus
management, 1:1 verification, 1:N identification, report-card generation aligned
with IREX I through X.

**Algorithms**
- Corpus with subject-id prefixing, genuine/impostor pair generation (impostor
  pairs capped at 1000) — `src/corpus.rs:83 genuine_pairs`,
  `src/corpus.rs:101 impostor_pairs`, prefix helper `corpus.rs:39 id_prefix`.
- 1:1 verification converting Hamming distance to similarity and applying a
  decision threshold — `src/verify.rs:40 IrexVerifier::verify`.
- 1:N identification with rank-N candidate lists —
  `src/identify.rs:38 IrexIdentifier::identify`.
- Metrics/report card: FMR/FNMR at threshold — `src/report.rs:41 fmr_fnmr`;
  ROC sweep (20 points), Rank-1 rate, per-group fairness slices assembled by
  `src/report.rs:59 ReportCard::generate`.

**Public API**

| Item | Kind |
|---|---|
| `IrexCorpus`, `CorpusEntry`, `CorpusError` | struct/enums (`corpus`) |
| `IrexVerifier`, `VerifyResult` | struct (`verify`) |
| `IrexIdentifier`, `IdentifyResult` | struct (`identify`) |
| `ReportCard`, `RocPoint`, `FairnessSlice` | structs (`report`) |

**Verification**: no `tests/` directory. 4 inline modules, 21 tests: corpus
rejects empty/duplicate ids; group filtering; genuine/impostor prefix rules and
1000-pair impostor cap; verifier scores, threshold flips, id propagation;
rank-1/rank-N ordering and caps; ROC has 20 points in range; fairness slices
cover all groups; report generation is panic-free on a small corpus.

## abis-iris-crosssensor

**Purpose**: "Cross-sensor iris template alignment": normalises templates
between NIR (700-1000 nm) and visible (400-700 nm) modalities via a learned
linear projection so cross-wavelength matching works without re-enrolment.

**Algorithms**
- Linear projection with identity weights plus a fitted per-dimension bias
  (mean gallery-minus-probe difference): `src/projection.rs:63
  LinearProjection::fit`, application `projection.rs:42 apply`. The doc comment
  calls this "the simplest least-squares solution when only a bias shift is
  assumed".
- Aligner wrapper over IrisCodes: bit-unpacking to f32 —
  `src/aligner.rs:87 iris_code_to_f32`; fit/transform delegation —
  `aligner.rs:52 CrossSensorAligner::fit`, `aligner.rs:75 transform`.

**Public API**

| Item | Kind |
|---|---|
| `CrossSensorAligner`, `AlignError`, `SensorType` | struct/enums (`aligner`) |
| `LinearProjection` | struct (`projection`) |
| `iris_code_to_f32` | fn (`aligner`) |

**Verification**: no `tests/` directory. 2 inline modules, 15 tests: identity
projection round-trips; fitting identical pairs gives near-zero bias; dimension/
empty-data errors; serde of `SensorType`; bit unpacking counts/values; Display
strings; fit-then-transform tolerance; NIR-to-visible identity alignment.

## abis-iris-fusion

**Purpose**: "Iris biometric score fusion: bilateral (left+right) and biometric
(iris+periocular)."

**Algorithms**
- Bilateral fusion of per-eye similarities (similarity = 1 - Hamming distance)
  with two strategies, `Average` and `LogicalAnd` (min), optional quality
  weighting where weights are proportional to template quality —
  `src/bilateral.rs:46 BilateralFusion::fuse`; single-eye helper
  `bilateral.rs:88 fuse_one`.
- Multibiometric weighted sum of iris and periocular scores; when one modality
  is missing its weight is folded into the present modality with a 0.5 factor —
  `src/biometric.rs:61 BiometricFusion::fuse`; weight validation
  `biometric.rs:25 validate`.

**Public API**

| Item | Kind |
|---|---|
| `BilateralFusion`, `BilateralResult`, `FusionStrategy` | struct/enums (`bilateral`) |
| `BiometricFusion`, `BiometricFusionResult`, `FusionWeights` | struct/enums (`biometric`) |

**Verification**: no `tests/` directory. 2 inline modules, 21 tests: identical
templates fuse near 1.0; scores in [0,1]; LogicalAnd bounded by weaker eye;
Average between eyes; occluded eye down-weighted; equal weights when weighting
disabled; weight validation; missing-modality fallbacks; exact weighted sums.

## abis-iris-motion

**Purpose**: "Motion blur estimation, multi-frame stacking, and iris identity
tracking."

**Algorithms**
- Blur estimation by Laplacian variance (L = 4c - left - right - top - bottom,
  clamped neighbours), normalised against a max variance of 1020 —
  `src/blur.rs:45 BlurEstimator::estimate`, `src/blur.rs:59 laplacian_variance`.
- Multi-frame stacking by integer mean averaging with rounding, u8-clamped —
  `src/stack.rs:29 FrameStacker::stack`.
- Identity tracking: accepts frames while centroid drift stays under
  `max_drift_px` and identity id is unchanged; drift or identity change breaks
  consistency and resets frame count — `src/tracker.rs:47 IrisTracker::update`.

**Public API**

| Item | Kind |
|---|---|
| `BlurEstimator`, `BlurResult` | structs (`blur`) |
| `FrameStacker`, `StackError` | struct, enum (`stack`) |
| `IrisTracker`, `TrackState`, `TrackerError` | struct/enums (`tracker`) |

**Verification**: no `tests/` directory. 3 inline modules, 22 tests: blur score
range and flat-vs-checkerboard discrimination; custom thresholds; stack
error/same-content/SNR-improvement/clamping behaviour; tracker first update,
drift rejection, identity-change resets, boundary-at-threshold consistency.

## abis-iris-pose

**Purpose**: "Off-axis gaze correction for iris images." Estimates gaze angle
from pupil/limbus displacement and warps the iris to compensate for off-axis
capture up to roughly +/-20 degrees.

**Algorithms**
- Pose estimate: gaze_x/gaze_y as pupil-offset divided by limbus radius, tilt
  from atan2 — `src/pose.rs:25 PoseCorrector::estimate`; correctability gate
  (normalised offset < 0.35 both axes) — `pose.rs:49 is_correctable`.
- Correction by integer pixel translation aligning pupil onto limbus centre,
  out-of-bounds samples zero-filled — `src/correction.rs:15
  OffAxisCorrector::correct`; post-check that corrected Hamming distance drops
  below 0.40 — `correction.rs:40 reduces_hd`.

**Public API**

| Item | Kind |
|---|---|
| `PoseCorrector`, `PoseEstimate` | structs (`pose`) |
| `OffAxisCorrector` | struct (`correction`) |

**Verification**: no `tests/` directory. 2 inline modules, 16 tests: frontal
gaze = 0; bounded components; finite/distinct tilt; correction preserves size,
identity for frontal, shifts off-axis, zero-fills out-of-bounds, no panics on
edges; correctability boundary; serde round-trip; zero-radius safety.

## abis-iris-lens

**Purpose**: "Contact lens detection for iris PAD (presentation attack
detection)." Detects transparent, coloured, and patterned lens artefact classes.
Cites Erdogmus & Marcel (2013) and LivDet-Iris 2017.

**Algorithms**
- Transparent lens: specular-ring analysis sampling 64 angular positions at
  1.05x limbus radius, confidence = fraction of samples brighter than 200 —
  `src/transparent.rs:15 detect_transparent`.
- Coloured lens: annulus homogeneity via mean absolute deviation
  (confidence = 1 - MAD/127.5) — `src/coloured.rs:13 detect_coloured`, pixel
  collection `coloured.rs:23 collect_annulus_pixels`.
- Patterned lens: radial strip sampled mid-annulus, 64-point DFT, confidence
  from AC peak vs DC magnitude — `src/patterned.rs:14 detect_patterned`, DFT at
  `patterned.rs:49 dft_magnitudes`.
- Combined detector with per-class thresholds returning `LensKind` /
  `LensResult` — `src/detector.rs:70 LensDetector::detect`.

**Public API**

| Item | Kind |
|---|---|
| `LensDetector`, `LensKind`, `LensResult`, `DetectError` | struct/enums (`detector`) |
| `detect_transparent`, `detect_coloured`, `detect_patterned` | fns |

**Verification**: no `tests/` directory. 4 inline modules, 24 tests: flat grey
-> no lens; bright ring -> transparent; homogeneous annulus -> coloured;
stripes -> patterned; undersized image errors; `LensKind` serde round-trip;
`has_lens` vs kind != None; confidence ranges; below-threshold gating; PAD
accept/reject mapping.

## abis-iris-dl

**Purpose**: "Deep iris embedding via hash-based deterministic embedder
(DeepIrisNet-inspired)" citing Gangwar & Joshi (ICIP 2016). Despite the name,
this crate contains **no neural network**: the shipped embedder is a SHA-256
hash expansion used as a "build-without-ONNX fallback".

**Algorithms**
- Hash embedder: 8 four-byte salts, each salt+pixels hashed with SHA-256, bytes
  mapped to signed floats, L2-normalised to 256-d —
  `src/hash_embedder.rs:38 HashEmbedder::embed`, `hash_embedder.rs:58
  l2_normalise`.
- Embedding container with cosine similarity and unit checks —
  `src/embedder.rs:26 Embedding::cosine_similarity`, `embedder.rs:17 l2_norm`.
- Facade embedder rejecting all-zero images — `src/embedder.rs:72
  IrisEmbedder::embed` (delegates to `HashEmbedder`).

**Public API**

| Item | Kind |
|---|---|
| `Embedding`, `EmbedError`, `IrisEmbedder` | struct, enum (`embedder`) |
| `HashEmbedder` | struct (`hash_embedder`) |

**Verification**: no `tests/` directory. 2 inline modules, 16 tests: norm/cosine
math including mismatch and zero-vector errors; embeddings are 256-d,
unit-norm, deterministic, distinct across images; serde round-trip.

---

## Honest closing notes

- **No real deep models anywhere.** `abis-iris-dl` ships only hash-based
  embedding; there is no ONNX/tch/tract dependency in any iris crate and no
  model-loading mechanism or `EEL_*_PATH` env var exists for iris. The
  DeepIrisNet reference describes intent, not implementation.
- **Cross-sensor alignment is bias-only.** `LinearProjection::fit` fits a
  per-dimension mean offset over an identity matrix, not a full learned
  projection; cross-wavelength claims rest on synthetic tests.
- **OSIRIS refinement is simplified**: `viterbi_refine` picks per-angle best
  radii and averages them — not a true Viterbi trellis despite the naming.
- **PIE is non-standard** (lib.rs calls it an "internal variant") and the IREX
  harness operates on already-encoded templates, not NIST data formats.
- **Test coverage is broad but synthetic.** All 200 inline tests run against
  constructed circles/rings/uniform images; no real sensor imagery, no
  integration tests, no fixtures.
