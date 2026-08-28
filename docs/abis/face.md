# Face Crates (`abis-face-*`)

Part of the ABIS service under `services/abis-svc/crates/`. Each directory is a
Cargo package named `eel-abis-<dir>` (e.g. `abis-face-align` =
`eel-abis-face-align`).

## Package map and source size

| Directory | Package | src lines | Inline test modules | Inline `#[test]` fns |
|---|---|---:|---:|---:|
| abis-face-detect | eel-abis-face-detect | 450 | 1 | 4 |
| abis-face-align | eel-abis-face-align | 428 | 0 | 0 |
| abis-face-embed | eel-abis-face-embed | 211 | 0 | 0 |
| abis-face-quality | eel-abis-face-quality | 577 | 0 | 0 |
| abis-face-occlusion | eel-abis-face-occlusion | 229 | 0 | 0 |
| abis-face-age | eel-abis-face-age | 186 | 0 | 0 |
| abis-face-kinship | eel-abis-face-kinship | 177 | 0 | 0 |
| abis-face-3dmm | eel-abis-face-3dmm | 310 | 0 | 0 |
| abis-face-crossdomain | eel-abis-face-crossdomain | 146 | 0 | 0 |
| abis-face-deident | eel-abis-face-deident | 325 | 0 | 0 |
| abis-face-video | eel-abis-face-video | 341 | 0 | 0 |
| abis-face-frvt | eel-abis-face-frvt | 403 | 0 | 0 |

Line counts are totals over each crate's `src/*.rs`.

**Correction to expectations:** these crates are *not* untested. Every one has
a `tests/` integration-test directory; inline `#[cfg(test)]` modules in `src`
are essentially absent (sole exception: `abis-face-detect`, 1 module / 4 tests
in `src/nms.rs`). Integration-test counts: detect 18, align 16, embed 20,
quality 32, occlusion 13, age 13, kinship 11, 3dmm 12, crossdomain 13,
deident 16, video 14, frvt 15.

## Interconnections

From the crates' `Cargo.toml` files: **none of the twelve depends on another
workspace crate.** Dependencies are `serde`, `thiserror`, `tracing`, plus
`sha2` in face-detect/embed/deident. Doc comments reference sibling
conventions — `FaceEmbedder` (`abis-face-embed/src/lib.rs:35`) expects input
"matching the output of `eel-abis-face-align`" (112x112 RGB) — but that
contract is convention only, not enforced by the build graph.

## abis-face-detect

**Purpose** (from `src/lib.rs`): "Face detection: integral-image Haar-like
baseline + optional ONNX sidecar" (Viola & Jones, CVPR 2001).

**Algorithms**
- Integral image with O(1) rectangle sums over u64 accumulators —
  `src/baseline.rs:24 IntegralImage::build`, `baseline.rs:43 rect_sum`,
  `baseline.rs:53 rect_mean`.
- Simplified four-stage cascade on window thirds (eye-band vs forehead,
  intensity floor, nose-bridge check, chin gradient), margin-derived confidence
  — `src/baseline.rs:68 passes_cascade`; multi-scale sliding-window detector
  implementing `FaceDetector` — `baseline.rs:160 impl FaceDetector for HaarDetector`.
- Greedy NMS — `src/nms.rs:12 nms`; IoU on `BoundingBox` (`lib.rs:35`); BT.601
  grayscale (`lib.rs:133 RgbImage::to_gray`). The baseline sets no landmarks;
  `FiveLandmarks` exists for richer detectors.

**Public API**

| Item | Kind |
|---|---|
| `FaceDetector` trait, `detect_gray` default | trait (`lib.rs`) |
| `HaarDetector` (with `with_min_size`, `with_conf_threshold`) | struct (`baseline`) |
| `nms` | fn (`nms`) |
| `BoundingBox` (+`iou`,`centre`,`area`), `FaceBox`, `FiveLandmarks`, `GrayImage`, `RgbImage`, `DetectError` | types |

**Verification**: essentially none inline — 1 `#[cfg(test)]` module, 4 `#[test]`
fns, all in `src/nms.rs` (keeps highest confidence; keeps non-overlapping;
empty input; single box). Real coverage: `tests/detect_test.rs` (203 lines,
18 integration tests) over cascade detection and NMS.

## abis-face-align

**Purpose**: "Face alignment: similarity transform mapping 5 detected landmarks
to the ArcFace 112x112 canonical template" (Deng et al., CVPR 2019 Appendix A;
Cootes Active Shape Models for the 68-point convention). Output size constant
`ALIGN_SIZE = 112`.

**Algorithms**
- Closed-form 2-D similarity fit via centroid normalisation and the 2x2
  cross-covariance; rotation = atan2(s10 - s01, s00 + s11), scale =
  trace(R^T S)/variance — `src/transform.rs:52 SimilarityTransform::fit`;
  wrappers `transform.rs:38 from_five`, `transform.rs:46 to_canonical`;
  apply/inverse at `transform.rs:132`/`:139`.
- ArcFace canonical five-point template — `src/canonical.rs:20 canonical_five`
  (exported as `CANONICAL_5`).
- Inverse-mapped bilinear warp into 112x112x3 — `src/warp.rs:73 warp_bilinear`,
  sampler `warp.rs:31 bilinear`.
- 68-point to five-point conversion — `src/landmarks.rs:91 Landmarks68::to_five`.

**Public API**

| Item | Kind |
|---|---|
| `SimilarityTransform` (+`from_five`,`to_canonical`,`fit`,`apply_point`,`inverse`), `AlignError` | struct, enum (`transform`) |
| `warp_bilinear`, `AlignedFace` | fn, struct (`warp`) |
| `Landmark`, `FiveLandmarks`, `Landmarks68` | structs (`landmarks`) |
| `canonical_five` / `CANONICAL_5`, `ALIGN_SIZE` | const/fn (`canonical`, `lib.rs`) |

**Verification**: zero inline test modules/functions. Coverage is entirely in
`tests/align_test.rs` (16 integration tests): transform fitting and warps.

## abis-face-embed

**Purpose**: "Face embedding: ArcFace/AdaFace/CosFace interface", citing Deng
(CVPR 2019), Kim (CVPR 2022) and Wang (CVPR 2018). Embedding dimension is fixed
at 512.

**Algorithms**
- Hash-based fallback embedder: SHA-256 seed followed by 64 iterated rounds
  hashing `(seed || round)` to produce 8 floats each, mapped to [-1, 1], then
  L2-normalised — `src/hash_embedder.rs:32 derive_raw`,
  `hash_embedder.rs:62 impl FaceEmbedder for HashEmbedder`.
- Vector math helpers: `l2_normalise` (`lib.rs:49`),
  `l2_normalise_slice` (`lib.rs:60`), `cosine_similarity` (`lib.rs:69`),
  `cosine_distance` (`lib.rs:74`), `is_normalised` (`lib.rs:79`).
- Model descriptor holding loss family plus an optional ONNX path —
  `src/model.rs:24 EmbeddingModel` with constructors `arcface` / `adaface` /
  `cosface` and `baseline(family)` for the no-file fallback.

**Public API**

| Item | Kind |
|---|---|
| `FaceEmbedder` trait (`embed`, `embed_vec`) | trait (`lib.rs`) |
| `HashEmbedder` | struct (`hash_embedder`) |
| `EmbeddingModel`, `LossFamily` | structs/enums (`model`) |
| `l2_normalise`, `l2_normalise_slice`, `cosine_similarity`, `cosine_distance`, `is_normalised` | fns (`lib.rs`) |
| `EMBED_DIM`, `EmbedError` | const, enum |

**Verification**: zero inline tests. `tests/embed_test.rs`: 20 integration tests (hash-embedder determinism, dimensionality, unit norm, similarity helpers).

## abis-face-quality

**Purpose**: "ISO/IEC 29794-5 face image quality assessment." Each metric is
independently computable, normalised to [0, 1] with 1.0 best.

**Algorithms**
- Sharpness by Laplacian variance (LAPV), clamped at an empirical max of 500 —
  `src/metrics.rs:17 sharpness`.
- Resolution: interocular distance vs the ISO 19794-5 minimum of 90 px —
  `src/metrics.rs:61 resolution_interocular`.
- Pose angles from landmark geometry — `src/metrics.rs:88 pose_angles`, scored
  at `metrics.rs:117 pose_quality`.
- Illumination (mean + range penalising extremes) — `metrics.rs:132`;
  border-ring background uniformity — `metrics.rs:161 uniformity`; skin-tone
  occlusion coverage — `metrics.rs:206 occlusion_coverage`; expression
  neutrality from mouth geometry — `metrics.rs:237`; gaze offset —
  `metrics.rs:263 gaze_offset`.
- Policy evaluation with violations + accept gate — `src/policy.rs:70 evaluate`,
  `policy.rs:131 accept`; report assembly `src/report.rs:37 compute_overall`,
  `report.rs:75 passes`, helper `policy.rs:137 make_report`.

**Public API**

| Item | Kind |
|---|---|
| `sharpness`, `resolution_interocular`, `pose_angles`, `pose_quality`, `illumination`, `uniformity`, `occlusion_coverage`, `expression_neutrality`, `gaze_offset` | fns (`metrics`, re-exported) |
| `QualityReport`, `PoseAngles` | structs |
| `QualityPolicy`, `PolicyViolation`, `make_report` | policy surface |
| `QualityLandmarks`, `QualityError` | types (`lib.rs`) |

**Verification**: zero inline tests. `tests/quality_test.rs`: 32 integration
tests (largest face suite) spanning metric ranges/boundaries and policy.

## abis-face-occlusion

**Purpose**: "Occlusion-robust face matching" citing Song et al. (ICCV 2019,
pairwise differential Siamese masking) and Zhao et al. (CVPR 2018,
attention-weighted embeddings). Single-file crate (`src/lib.rs`).

**Algorithms**
- Occlusion detection by skin-tone presence in three horizontal bands:
  bottom low skin => Mask, middle => Glasses, top => Hijab — `OcclusionDetector::detect`
  (src/lib.rs ~:70); predicate `R>G>B, R in [80,240], R-B>15`; overall fraction
  via `occlusion_fraction`.
- Attention-gated matching: occluded bands down-weight the matching embedding
  thirds (170 dims each of 512) via `build_gate`; similarity on gated vectors
  via `masked_similarity` (default threshold 0.5).

**Public API**

| Item | Kind |
|---|---|
| `OcclusionDetector` | struct |
| `OcclusionRobustMatcher` (+`build_gate`, `masked_similarity`) | struct |
| `OcclusionRegion`, `OcclusionType`, `OcclusionError` | enum/type |

**Verification**: zero inline tests; 13 integration tests in `tests/occlusion_test.rs`.

## abis-face-age

**Purpose**: "Age-invariant face matching" (Guo survey 2019; Wang et al. CVPR
2022 residual age-normalisation). Single file, 186 lines.

**Algorithms**
- Hash-based deterministic age estimator mapping embedding SHA-256 to an age in
  [15, 79] with mid confidence — `HashAgeEstimator::estimate` (`AgeEstimator` trait).
- Age-invariant matching: project out a stubbed 512-d "age direction" (unit
  vector along dim 0, lambda 0.3) then cosine — `AgeInvariantMatcher::deage`
  / `similarity`.
- Age-progression **stub**: perturbs every third dimension by a years-scaled
  epsilon — `AgeProgressionStub::progress`; docs state real progression needs a
  generative model.

**Public API**

| Item | Kind |
|---|---|
| `AgeEstimator` trait, `HashAgeEstimator` | trait, struct |
| `AgeInvariantMatcher` (+`with_lambda`, `similarity`) | struct |
| `AgeProgressionStub` (+`progress`) | struct |
| `AgeEstimate`, `AgeError` | type, enum |

**Verification**: zero inline tests; 13 integration tests in `tests/age_test.rs`.

## abis-face-kinship

**Purpose**: "Kinship verification and family cluster grouping" (Lu et al.
BMVC 2014; Guo & Wang CVPR 2012). The doc comment admits kinship here is
estimated purely from embedding cosine similarity — an explicit baseline, not a
learned kinship metric. Single file, 177 lines.

**Algorithms**
- Pairwise classification from cosine thresholds (defaults kin 0.5, twin 0.95)
  mapping similarity bands to Parent/Sibling/Twin/Other and a linear p_kin —
  `KinshipClassifier::predict`.
- Family grouping by union-find over pairwise kin predictions —
  `cluster_families` (recursive find with path compression).

**Public API**

| Item | Kind |
|---|---|
| `KinshipClassifier` (+`predict`) | struct |
| `cluster_families` | fn |
| `FamilyCluster`, `KinshipPrediction`, `Relationship`, `KinshipError` | types |

**Verification**: zero inline tests; 11 integration tests in `tests/kinship_test.rs`.

## abis-face-3dmm

**Purpose**: "3D Morphable Model (3DMM) interface for face reconstruction and
PAD" citing Blanz & Vetter (SIGGRAPH 1999), Basel Face Model (Paysan 2009),
Deng 3D reconstruction (CVPRW 2019).

**Algorithms**
- Pose representation and frontal checks — `src/bfm.rs:79 Pose3d` (`frontal()`,
  `is_near_frontal()`).
- `Model3dmm` trait (fit from landmarks, render from pose) — `bfm.rs:150`; only
  implementation is `StubModel3dmm` (`bfm.rs:178`): roll from the first two
  landmarks, yaw/pitch from centroid offsets vs hardcoded 56 px centres,
  all-zero shape/expression coefficients; render returns uniform grey.
- Geometric PAD: attack flagged when recovered pose deviates beyond yaw/pitch
  thresholds — `src/spoof.rs:55 SpoofDetector::check`,
  `spoof.rs:75 check_frontal`.

**Public API**

| Item | Kind |
|---|---|
| `Model3dmm` trait (`fit`, `render`) | trait (`bfm`) |
| `StubModel3dmm` | struct (`bfm`) |
| `Fit3dmm` (+`identity`, `to_frontal`), `Pose3d`, `Camera3d`, `Landmark`, `DmmError` | types (`bfm`) |
| `SpoofDetector` (+`check`, `check_frontal`), `SpoofDecision` | spoof surface |

**Verification**: zero inline tests; 12 integration tests in `tests/dmm_test.rs`.

## abis-face-crossdomain

**Purpose**: "Cross-domain (visible / NIR / thermal) face embedding alignment",
heterogeneous FR after Klare & Jain (TPAMI 2013). Single file, 146 lines.

**Algorithms**
- Cross-modality aligner fitted from paired embeddings. Despite doc language
  about least-squares P = Y X^+, the implementation fits only a per-dimension
  mean-ratio scale and re-normalises on transform — `CrossDomainAligner::fit`
  / `transform`, mirrored through the `CrossDomainAlign` trait.

**Public API**

| Item | Kind |
|---|---|
| `CrossDomainAligner` (+`new`, `fit`, `transform`, `is_fitted`) | struct |
| `CrossDomainAlign` trait | trait |
| `Modality` (Visible/Nir/Thermal), `CrossDomainError` | enums |

**Verification**: zero inline tests; 13 integration tests in `tests/crossdomain_test.rs`.

## abis-face-deident

**Purpose**: "Privacy-preserving face de-identification" citing Sweeney
k-anonymity (2002), Dwork DP (2006), Gross composite faces (2009).

**Algorithms**
- k-anonymity: replace target pixels with the pixel-wise mean of the k most
  similar anonymity-set members by embedding cosine — `src/kanon.rs:57
  KAnonDeident::deidentify`; SHA-256 helper `kanon.rs:99 embedding_hash`.
- Differential privacy: Gaussian mechanism, sigma = sensitivity *
  sqrt(2 ln(1.25/delta)) / epsilon with sensitivity 2 for unit-norm embeddings
  — `src/dp.rs:60 sigma`; Box-Muller noise on a hash-based counter PRNG —
  `dp.rs:70 add_noise` (documented CI-deterministic; use a CSPRNG in production).
- Pixel privacy: box blur and pixelation — `src/blur.rs:10 blur_image`,
  `blur.rs:50 pixelate_image`.

**Public API**

| Item | Kind |
|---|---|
| `KAnonDeident` (+`deidentify`), `KAnonError` | struct, enum (`kanon`) |
| `DpEmbeddingNoise` (+`new`, `sigma`, `add_noise`), `DpError` | struct, enum (`dp`) |
| `blur_image`, `pixelate_image` | fns (`blur`) |
| `embedding_hash` | fn (`kanon`) |

**Verification**: zero inline tests; 16 integration tests in `tests/deident_test.rs`.

## abis-face-video

**Purpose**: "Multi-frame face tracking and quality-weighted embedding fusion."
Cites SORT (Bewley et al., ICIP 2016) and Best & Best FG 2021 for fusion.

**Algorithms**
- SORT-inspired tracker: greedy assignment scoring 0.6 * IoU + 0.4 * embedding
  cosine (neutral 0.5 when embeddings absent), miss counters, pruning after
  `max_missed` frames — `src/tracker.rs:55 Tracker::feed_frame`; box IoU at
  `src/track.rs:22 BBox::iou`.
- Track bookkeeping (detections, last bbox/embedding, frame count) —
  `src/track.rs:80 Track`.
- Fusion: f64 quality-weighted mean of track embeddings, L2-normalised —
  `src/fusion.rs:25 fuse_track`.

**Public API**

| Item | Kind |
|---|---|
| `Tracker` (+`with_thresholds`, `feed_frame`, `active_tracks`) | struct (`tracker`) |
| `Track`, `FrameDetection`, `TrackUpdate`, `TrackId`, `BBox` | types (`track`) |
| `fuse_track`, `FusedTemplate` | fn, struct (`fusion`) |

**Verification**: zero inline tests; 14 integration tests in `tests/video_test.rs`.

## abis-face-frvt

**Purpose**: "NIST FRVT evaluation harness": 1:1 FMR/FNMR, 1:N rank
identification, quality-FNMR correlation, per-demographic fairness slices.

**Algorithms**
- Verification metrics: FMR/FNMR from genuine/impostor cosine pairs at a
  threshold — `src/metrics.rs:29 FmrFnmr::compute` (similarity helper
  `metrics.rs:8 cosine_sim`).
- Identification: Rank-N hit rate over gallery/probe sets split by id prefix —
  `src/metrics.rs:73 RankN::compute`.
- Quality utility: Pearson correlation between quality score and FNMR outcome —
  `src/metrics.rs:128 QualityCorrelation::compute`.
- Fairness slices by self-reported demographics (FMR/FNMR per SkinTone) —
  `src/metrics.rs:180 FairnessSlice::compute_by_skin_tone`.
- Corpus with demographic tagging and pair generation —
  `src/corpus.rs:102 FrvtCorpus::genuine_pairs`, `corpus.rs:116 impostor_pairs`.
- Report aggregation and log summary — `src/report.rs:31 FrvtReport::summary`.

**Public API**

| Item | Kind |
|---|---|
| `FrvtCorpus`, `FrvtImage`, `FrvtImageSet`, `SkinTone`, `Sex` | corpus surface |
| `FmrFnmr`, `RankN`, `QualityCorrelation`, `FairnessSlice` | metric structs |
| `FrvtReport` (+`summary`) | report |

**Verification**: zero inline tests; 15 integration tests in `tests/frvt_test.rs`.

---

## Honest closing notes

- **Runtime model dependencies live outside Rust.** No crate loads weights.
  The `onnx` features in `face-detect`/`face-embed` are empty markers
  (`onnx = []`); inference is meant to go through the Python sidecar
  `py/face-onnx-worker`, which reads `EEL_FACE_DETECT_ONNX_PATH`,
  `EEL_FACE_LANDMARK_ONNX_PATH`, `EEL_FACE_EMBED_ONNX_PATH` and
  `EEL_FACE_QUALITY_ONNX_PATH` (`eel_face_worker/inference.py:4-7`). With no
  sidecar, the Rust crates run the heuristics described above.
- **Hash embedders are placeholders.** `abis-face-embed/src/hash_embedder.rs`
  derives 512-d vectors from iterated SHA-256 of pixel bytes. Deterministic and
  fast, but it encodes no facial semantics; match scores from it are
  meaningless beyond CI plumbing. Same pattern as the iris `HashEmbedder`.
- **Explicit stubs.** `StubModel3dmm` renders flat grey with zero coefficients;
  `AgeProgressionStub` is numerical only; `HashAgeEstimator` fabricates ages.
  Kinship and cross-domain docs admit baseline status (cosine thresholds;
  mean-ratio scaling instead of least-squares projection).
- **Heuristic detectors.** `HaarDetector` is a four-rule brightness cascade,
  not trained features; occlusion logic shares one crude RGB skin predicate
  that may misfire across skin tones/lighting — relevant to the fairness
  ambitions of `abis-face-frvt`.
- **No intra-crate wiring.** The detect -> align -> embed pipeline exists only
  in doc comments; Cargo ties nothing together, so format drift between crates
  would not be caught at compile time.
- **Testing posture differs from the iris side**: logic tests live in
  `tests/` as integration suites with almost no inline unit tests (NMS only).
  All assertions use synthetic images; no real imagery fixtures or golden-score
  regressions exist.
