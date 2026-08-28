# Fingerprint Technology Area (ABIS)

Documentation for the fingerprint crates under `services/abis-svc/crates/`.
Paths cited are relative to the repository root; claims verified against source.

## Package name mapping

Each directory publishes one crate prefixed `eel-`; the `[lib]` name replaces
hyphens with underscores.

| Directory | Package name |
|---|---|
| `abis-fp-mcc` | `eel-abis-fp-mcc` |
| `abis-fp-skeleton` | `eel-abis-fp-skeleton` |
| `abis-fp-ridge-count` | `eel-abis-fp-ridge-count` |
| `abis-fp-orientation` | `eel-abis-fp-orientation` |
| `abis-fp-level3` | `eel-abis-fp-level3` |
| `abis-fp-texture` | `eel-abis-fp-texture` |
| `abis-fp-tps-warp` | `eel-abis-fp-tps-warp` |
| `abis-fp-consensus` | `eel-abis-fp-consensus` |
| `abis-fp-multifinger` | `eel-abis-fp-multifinger` |
| `abis-fp-nfiq22` | `eel-abis-fp-nfiq22` |
| `abis-fp-cdeffs` | `eel-abis-fp-cdeffs` |
| `abis-fp-dl-onnx` | `eel-abis-fp-dl-onnx` |
| `abis-fp-fpvte` | `eel-abis-fp-fpvte` |
| `abis-fp-sourceafis-bridge` | `eel-abis-fp-sourceafis-bridge` |
| `abis-enhancement` | `eel-abis-enhancement` |
| `abis-rectification` | `eel-abis-rectification` |

## Line-count inventory

Totals of all `src/*.rs` and `tests/*.rs` per crate.

| Crate | src | test | tests (`#[test]`) |
|---|---|---|---|
| abis-fp-mcc | 416 | 385 | 25 |
| abis-fp-skeleton | 715 | 250 | 15 |
| abis-fp-ridge-count | 408 | 266 | 17 |
| abis-fp-orientation | 517 | 318 | 22 |
| abis-fp-level3 | 591 | 270 | 18 |
| abis-fp-texture | 763 | 319 | 25 |
| abis-fp-tps-warp | 577 | 261 | 15 |
| abis-fp-consensus | 516 | 412 | 23 |
| abis-fp-multifinger | 557 | 271 | 24 |
| abis-fp-nfiq22 | 598 | 176 | 13 |
| abis-fp-cdeffs | 845 | 307 | 25 |
| abis-fp-dl-onnx | 462 | 191 | 16 |
| abis-fp-fpvte | 637 | 266 | 18 |
| abis-fp-sourceafis-bridge | 726 | 243 | 17 |
| abis-enhancement | 547 | 160 | 15 |
| abis-rectification | 324 | 147 | 14 |
| **Total** | **9,299** | **4,242** | **302** |

All tests are integration tests in a single `tests/*.rs` per crate; none of
these sixteen crates contain inline `#[cfg(test)]` modules. (Adjacent,
out-of-scope `abis-fp-vendor-peak` does.)

## Interconnection map

From the `Cargo.lock` dependency graph; no other workspace crate consumes
these except as listed.

```
eel-abis-fp-consensus ──> eel-abis-fp-mcc / -texture / -orientation / -skeleton
                      ──> abis-fusion-advanced   (weighted_vote)
                      ──> abis-matchers Bozorth3 (abis_matcher::matcher::fingerprint_matcher)

eel-abis-fp-vendor-peak ──> eel-abis-fp-consensus

FingerprintTemplate/Minutia shared types from abis-matcher, consumed by:
    abis-fp-mcc, abis-fp-ridge-count, abis-fp-level3, abis-fp-cdeffs
```

Structural facts:

- All other matcher crates are leaf libraries; nothing wires `abis-fp-nfiq22`,
  `abis-fp-multifinger`, `abis-fp-dl-onnx`, `abis-fp-fpvte`, `abis-fp-tps-warp`,
  `abis-fp-sourceafis-bridge`, `abis-enhancement`, or `abis-rectification`
  into a service binary yet.
- Each image-consuming crate defines its own `GrayImage`
  (`services/abis-svc/crates/abis-fp-texture/src/lib.rs:27`,
  `abis-fp-orientation/src/lib.rs:24`, `abis-fp-nfiq22/src/lib.rs:21`,
  `abis-fp-dl-onnx/src/lib.rs:23`, `abis-fp-sourceafis-bridge/src/lib.rs:29`,
  `abis-fp-multifinger/src/lib.rs:24`) plus `GrayscaleImage` in
  `abis-enhancement/src/fingerprint.rs:6`; no shared image type exists.
- `abis-rectification` depends on no biometric crate.

## abis-fp-mcc

**Purpose.** Minutia Cylinder-Code matching per Cappelli/Ferrara/Maltoni IEEE TPAMI 2010: one binary cylinder per minutia, matched by consolidation.

**Algorithms.** Cylinder bit-coding on an `Ns x Ns` grid rotated to the minutia direction across `Nd` directional bins; a bit is set when accumulated Gaussian spatial/directional contributions exceed `mu_psi` (`services/abis-svc/crates/abis-fp-mcc/src/build.rs:44 build_cylinder`, bit-pack at `build.rs:89`, contribution sum per Cappelli eq. (3) at `build.rs:114 sum_contributions`); invalid cylinders (`tau_psi`) excluded from consolidation (`build.rs:98`). Local similarity = popcount-normalised dot product (LSS, eq. (8), `src/similarity.rs:13 local_similarity`); global LSSR = greedy one-to-one selection over sorted pair similarities gated by `min_matched` (`src/consolidate.rs:19 global_similarity`).

**Public API.**

| Item | Signature |
|---|---|
| `build_mcc` | `fn build_mcc(template: &FingerprintTemplate, params: &MccParams) -> Vec<MinutiaCylinder>` |
| `local_similarity` / `global_similarity` | see Algorithms |
| `MccMatcher` | `new(MccParams)`; `compare(&self, probe, gallery: &FingerprintTemplate) -> f32` |
| `MinutiaCylinder`, `MccParams` | data types (`cells: Vec<u8>`, `popcount`, `valid`) |

**Verification.** 25 tests (`tests/mcc_test.rs`): cylinder count/bit length vs parameters; identical templates score 1.0; 5/15-degree rotation tolerated; +/-5-minutiae perturbations acceptable while disjoint templates score low; symmetry and clamping.

---

## abis-fp-skeleton

**Purpose.** Ridge skeleton extraction and graph matching: binarise, Zhang-Suen thinning, ridge graph, subgraph-isomorphism matching.

**Algorithms.** Zhang-Suen thinning with two deletion subpasses (`services/abis-svc/crates/abis-fp-skeleton/src/skeleton.rs:77 extract_skeleton`; conditions at `skeleton.rs:162/168/179/185`). Ridge graph with endpoints/bifurcations as nodes and traced segments (length, curvature, orientation) as edges (`src/graph.rs:66 skeleton_to_graph`, tracing at `graph.rs:132 trace_edge`). Matching via VF2-style edge coverage for small graphs, greedy for large (`src/matcher.rs:143 vf2_edge_coverage`, `matcher.rs:123 greedy_edge_coverage`).

**Public API.**

| Item | Signature |
|---|---|
| `extract_skeleton` | `fn extract_skeleton(image: &BinaryImage) -> Result<RidgeSkeleton, SkeletonError>` |
| `skeleton_to_graph` | `fn skeleton_to_graph(skel: &RidgeSkeleton) -> RidgeGraph` |
| `SkeletonMatcher` | `new(threshold)`; `compare(probe, gallery: &BinaryImage)`; `compare_graphs(...)` |
| `RidgeGraph`, `RidgeNode`, `RidgeEdge`, `NodeKind`, `connected_components` | data types/helpers |

**Verification.** 15 tests (`tests/skeleton_test.rs`): solid rectangle thins to single-pixel width; line endpoints become nodes; identical thin-line images score high; ~10% missing-edge tolerance.

---

## abis-fp-ridge-count

**Purpose.** Bozorth3-style inter-minutia ridge counting plus K-plet descriptor matching (NIST NBIS references).

**Algorithms.** Ridge counting integrates crossing density along the straight path between minutiae using the orientation field: per-step crossings = `|sin(path_angle - ridge_orientation)| / INTER_RIDGE_SPACING_PX` (15 px), with geometric fallback when no field is given (`services/abis-svc/crates/abis-fp-ridge-count/src/count.rs:79 count_ridges`). K-plets: K=5 nearest neighbours per minutia as tuples of distances, angles, ridge counts (`src/kplet.rs:49 build_kplets`). Tuple matching under tolerances rho=15 px / phi=20 deg / theta=15 deg / ridge-count tol 2 (`src/matcher.rs:20 tuple_similarity`).

**Public API.**

| Item | Signature |
|---|---|
| `count_ridges` | `fn count_ridges(from: &Minutia, to: &Minutia, orientation_field: &OrientationField) -> u8` |
| `build_kplets` | builds K=5 tuples per minutia |
| `KPletMatcher` / `RidgeCountMatcher` | topology and ridge-count-agreement matchers |

**Verification.** 17 tests (`tests/ridge_count_test.rs`): zero count for coincident minutiae; monotonic in distance; field sampling clamps; both matchers score identical templates at 1.0; different topologies distinguished.

---

## abis-fp-orientation

**Purpose.** Orientation-field analysis, Poincare singular-point detection, Henry pattern classification, multi-granularity matcher.

**Algorithms.** Block-wise structure-tensor orientation field (Rao-style gradients): angle = `0.5*atan2(vy, vx)` with coherence (`services/abis-svc/crates/abis-fp-orientation/src/field.rs:53 compute_orientation_field`); doubled-angle correlation `cos(2*(a-b))` (`field.rs:113 field_correlation`). Poincare-index singular points (`src/singular.rs:42 detect_singular_points`, index at `singular.rs:89`). Pattern classification {Arch..Whorl, TwinLoop} from core/delta counts and positions (`src/pattern.rs:43 classify_pattern`). Coarse-to-fine matcher: class gate, singular agreement, field correlation (`src/matcher.rs:57 compare`, `matcher.rs:108 singular_point_score`).

**Public API.**

| Item | Signature |
|---|---|
| `compute_orientation_field` | `fn compute_orientation_field(image: &GrayImage, block_size: u32) -> Result<OrientationField, OrientationError>` |
| `field_correlation` | `fn field_correlation(a: &OrientationField, b: &OrientationField) -> Result<f32, OrientationError>` |
| `detect_singular_points` / `classify_pattern` | see Algorithms |
| `OrientationMatcher` | `new(block_size, threshold)`; `compare(probe, gallery: &GrayImage) -> Result<OrientationScore, _>` |

**Verification.** 22 tests (`tests/orientation_test.rs`): striped synthetic images give correct angles; identical fields correlate at 1.0; flat images yield no singular points; arch/tented-arch/loop/whorl classifications correct; partial rotational invariance.

---

## abis-fp-level3

**Purpose.** Level-3 features at >= 500 ppi: sweat pores, ridge-edge contours, incipient ridges (Jain/Chen/Demirkus TPAMI 2007).

**Algorithms.** Pores as 8-connected local minima with contrast confidence, non-maximum suppression at 8 px spacing (`services/abis-svc/crates/abis-fp-level3/src/pores.rs:38 extract_pores`, NMS at `pores.rs:111`). Ridge edges with curvature (`src/ridge_edge.rs:28 extract_ridge_edges`, curvature at `ridge_edge.rs:94`). Incipient ridges via <=60 px length, <=5 px width, >=0.08 contrast gates (`src/incipient.rs:37 detect_incipient_ridges`). Hard resolution gate `MIN_RESOLUTION_PPI = 500` (`src/quality.rs:11`). Pore-constellation matching: k-nearest (K=5) pores around corresponding minutiae, centroid/RMS-normalised shape similarity plus edge bonus capped at 0.15 (`src/matcher.rs:42 compare`, `matcher.rs:111 pore_constellation_similarity`).

**Public API.**

| Item | Signature |
|---|---|
| `extract_pores` | `fn extract_pores(intensity: &[u8], width: usize, height: usize, orientation_field: &[u8]) -> Vec<Pore>` |
| `extract_ridge_edges` / `detect_incipient_ridges` / `Level3Quality::check_resolution` | extractors and gate |
| `Level3Matcher::compare` | `fn compare(&self, probe, gallery: &Level3Features, probe_minutiae, gallery_minutiae: &[(f32,f32)]) -> Result<f32, Level3QualityError>` |

**Verification.** 18 tests (`tests/level3_test.rs`): 300-ppi input rejected; pore confidence bounded and post-NMS spacing respected; mismatched buffers yield empty results; identical pore constellations score high.

---

## abis-fp-texture

**Purpose.** Texture fallback matching when minutiae quality is low: FingerCode, LBP, MB-LBP, fused.

**Algorithms.** FingerCode: reference point via core detection (`fingercode.rs:83 locate_reference_point`, Poincare at `:163`); 80-sector tessellation (16 angular x 5 bands, `:188 tessellate`); 8-filter 15x15 Gabor bank (`:230 filter_bank`, kernel `:242 gabor_filter`); mean-absolute-deviation encoding (640-d, `:294 encode_fingercode`); Euclidean-style match (`:327 match_fingercode`). LBP: 256-bin histogram, chi-square distance (`src/lbp.rs:17 lbp_at`, `lbp.rs:80 chi_square_distance`). MB-LBP: block-mean LBP at three scales, 768-d multi-scale histogram (`src/mb_lbp.rs:51 mb_lbp_at`, `:103 mb_lbp_multi_scale`). Weighted fusion in `src/matcher.rs TextureMatcher`.

**Public API.**

| Item | Signature |
|---|---|
| FingerCode pipeline | `locate_reference_point`, `tessellate`, `filter_bank`, `encode_fingercode`, `match_fingercode(a, b: &FingerCode) -> f32` |
| LBP / MB-LBP pipelines | `lbp_histogram_for_image`, `chi_square_distance`, `mb_lbp_histogram`, `mb_lbp_multi_scale`, `mb_lbp_similarity` |
| `TextureMatcher` | `new(weights: TextureWeights, threshold: f32)`; `compare(...) -> TextureScoreBreakdown`; `is_match(...)` |

**Verification.** 25 tests (`tests/texture_test.rs`): exactly 80 in-bounds sectors; filter bank yields 8 non-negative same-sized images; FingerCode length 640 with self-match 1.0; histograms normalised; MB-LBP length 768; fused matcher ranks identical above different.

---

## abis-fp-tps-warp

**Purpose.** Thin-Plate Spline non-rigid distortion correction applied before minutiae matching (Bookstein 1989).

**Algorithms.** TPS fit: (N+3)^2 linear system combining the U(r) radial basis (`tps_kernel`, `tps.rs:18`) with affine terms, solved by Gaussian elimination with partial pivoting (`services/abis-svc/crates/abis-fp-tps-warp/src/tps.rs:43 TpsWarp::fit`, solver `tps.rs:148 gaussian_solve`); evaluation at `tps.rs:119 apply`. Registration: best rigid alignment, correspondence filtering (50 px / 0.35 rad, min 3 pairs), TPS fit from correspondences (`src/register.rs:21 register_minutiae`). Post-warp overlap scoring (`src/matcher.rs:11 minutiae_overlap_score`).

**Public API.**

| Item | Signature |
|---|---|
| `TpsWarp` | `fit(source, target: &[Point]) -> Result<TpsWarp, TpsError>`; `apply(x, y) -> (f64, f64)`; `identity()` |
| `register_minutiae` | fits probe->gallery TPS; returns `Option<TpsWarp>` |
| `minutiae_overlap_score` | `fn minutiae_overlap_score(probe: &[Minutia], gallery: &[Minutia]) -> f32` |
| `TpsWarpMatcher` / `TpsMatchResult` / `Minutia::warped` | warp-then-match wrapper; warping keeps theta (radians) |

**Verification.** 15 tests (`tests/tps_test.rs`): <3 points and count mismatches error; source==target gives identity; control points map exactly; synthetic barrel distortion corrected; warping preserves theta; matcher maintains-or-improves score versus unwarped.

---

## abis-fp-consensus

**Purpose.** Multi-algorithm consensus matcher: up to five sub-matchers fused by weighted voting (Jain/Nandakumar/Ross normalisation reference).

**Algorithms.** Orchestration of Bozorth3 (from `abis-matcher`), MCC, FingerCode texture, orientation, and skeleton sub-matchers; each raw score passes a Platt sigmoid calibrator and a per-matcher threshold (`services/abis-svc/crates/abis-fp-consensus/src/consensus.rs:113 ConsensusMatcher::compare`; thresholds `consensus.rs:66-70`; weights `[1.0,1.2,0.8,0.7,0.9]` at `:73`). Decision = min-agreement policy AND weighted vote via `abis-fusion-advanced::decision_level::weighted_vote` (`consensus.rs:222-243`). Platt calibration by gradient descent (`src/calibration.rs:46 fit`). MinAgreementPolicy default 3-of-5, scaling down for fewer matchers (`src/policy.rs:30 check`).

**Public API.**

| Item | Signature |
|---|---|
| `ConsensusMatcher::compare` | `fn compare(&self, input: &ConsensusInput) -> Result<ConsensusScore, ConsensusError>` |
| `ConsensusInput<'a>` / `ConsensusScore` | optional minutiae/image/binary slices; `fused`, `per_matcher`, `decision`, `explanation` |
| `ScoreCalibrator` | `identity()`, `new(a,b)`, `calibrate(raw)`, `fit(scores, labels)` |
| `MinAgreementPolicy` | `new(min_votes)`, `check(votes, total)` |

**Verification.** 23 tests (`tests/consensus_test.rs`): policy defaults and scaling; calibrator rejects <4 samples and detects non-monotone fits; minutiae-only input runs exactly Bozorth3+MCC; all five run with full input; 1-of-5 agreement rejected; explanations include votes.

---

## abis-fp-multifinger

**Purpose.** Slap segmentation (4-4-2 acquisition), per-finger weighted fusion, detection-per-request (DPR) gating (ISO/IEC 29109-2 framing).

**Algorithms.** Otsu binarisation (`slap.rs:75 otsu_threshold`), vertical projection (`slap.rs:121`), valley detection (`slap.rs:134 find_valleys`), column segmentation with even-split fallback (`services/abis-svc/crates/abis-fp-multifinger/src/slap.rs:161 segment_slap`). Position-weighted fusion, thumb highest / little lowest (`src/multifinger.rs:45 position_weight`, `:56 fuse_per_finger`). DPR gate: reject when fewer than `min_matched` (2) of `min_available` (6) fingers exceed threshold (`src/dpr.rs DprPolicy::evaluate`). Per-finger similarity is currently a Bhattacharyya coefficient between sampled intensity histograms, not a minutia match (`src/matcher.rs:24 image_similarity`) — see Notes.

**Public API.**

| Item | Signature |
|---|---|
| `segment_slap` | `fn segment_slap(image: &GrayImage, kind: SlapKind) -> Result<Vec<(FingerPosition, BoundingBox, GrayImage)>, MultiFingerError>` |
| `otsu_threshold` / `DprPolicy::evaluate` | `fn otsu_threshold(pixels: &[u8]) -> u8`; `evaluate(matched, available) -> DprResult` |
| `position_weight` / `fuse_per_finger` | fusion helpers |
| `MultiFingerMatcher` | `new(per_finger_threshold)`; `compare_slaps(...)`; `compare_per_finger_scores(...)` |

**Verification.** 24 tests (`tests/multifinger_test.rs`): correct segment counts/positions for right-four/left-four/thumbs; non-overlapping boxes covering full width; Otsu on bimodal/uniform images; weight ordering thumb > index > middle > little; DPR rejects 0-1 of 6, passes 2 of 6, with reasons.

---

## abis-fp-nfiq22

**Purpose.** NFIQ 2.2 quality assessment: 14 features + Random Forest per NIST SP 500-333; output integer [0,100], higher better.

**Algorithms.** Feature extraction incl. Local Clarity Score, Orientation Certainty Level (Sobel-based), Orientation Flow, ROI, FDA, Gabor response (`services/abis-svc/crates/abis-fp-nfiq22/src/features.rs:134 extract_features`, Sobel at `:100`, OCL at `:116`). Random forest of JSON trees, prediction = mean tree output clamped [0,100] (`src/model.rs:44 DecisionTree::predict`, `model.rs:74 RandomForestModel::predict`). `nfiq22_score` uses a built-in minimal *synthetic* default model; production should load the NIST model via `RandomForestModel::from_path` (`src/scorer.rs:14`, `scorer.rs:20 score_with_model`). Raw-score calibration in `src/calibrator.rs Calibrator`.

**Public API.**

| Item | Signature |
|---|---|
| `extract_features` | `fn extract_features(image: &GrayImage) -> Result<Nfiq22Features, Nfiq22Error>` |
| `nfiq22_score` / `score_with_model` | `-> Result<u8, Nfiq22Error>`; the latter takes `&RandomForestModel` |
| `RandomForestModel` | JSON ensemble; `predict`, `from_path` |
| `Calibrator` | `from_raw_scores`, `fit(images, model)`, `calibrate(raw) -> u8` |

**Verification.** 13 tests (`tests/nfiq22_test.rs`): all 14 features finite; tiny images rejected; striped textures outscore solids and blurred variants; RF output bounded; calibrator shifts distributions. Synthetic-data sanity only.

---

## abis-fp-cdeffs

**Purpose.** CDEFFS Extended Feature Set records (NIST SP 500-290 e3 Annex A; ANSI/NIST-ITL Type-9 fields 300-999): parse, serialise, fuse into matching.

**Algorithms.** TLV wire format: u16 BE tag + u16 BE length + payload, tags 300+ (`services/abis-svc/crates/abis-fp-cdeffs/src/parse.rs:42 parse_efs`; `src/serialise.rs:17 serialise_efs`). Data model spans pattern class, singular points, ridge-quality/flow/wavelength maps, segmentation mask, extended minutiae, dots, incipient ridges, skin condition, ridge-edge shapes (`src/features.rs:186 ExtendedFeatureSet`). Matcher contributes a clamped delta to a base score: incompatible classes hard-reject, class agreement bonus, singular-point agreement (20 px / 20 deg) up to +0.05 (`src/matcher.rs:35 CdeffsMatcher::fuse`, `:60 class_score`, `:94 singular_point_score`).

**Public API.**

| Item | Signature |
|---|---|
| `parse_efs` | `fn parse_efs(bytes: &[u8]) -> Result<ExtendedFeatureSet, ParseError>` |
| `serialise_efs` | `fn serialise_efs(efs: &ExtendedFeatureSet) -> Vec<u8>` |
| `CdeffsMatcher::fuse` | `fn fuse(&self, base_score: f32, probe, gallery: &ExtendedFeatureSet) -> f32` |

**Verification.** 25 tests (`tests/cdeffs_test.rs`): byte-identical round trip incl. empty EFS; every field parses back correctly (classes, cores/deltas, maps, extended minutiae, dots, incipient ridges, skin condition, edge shapes); class mismatches penalised, singular agreement rewarded, scores clamped.

---

## abis-fp-dl-onnx

**Purpose.** DeepPrint-style deep embedding (Engelsma/Cao/Jain TPAMI 2021) with ONNX Runtime behind an opt-in feature, plus fusion with classical matchers.

**Algorithms.** Preprocessing: bilinear resize to 224x224, normalise [-1,1] (`services/abis-svc/crates/abis-fp-dl-onnx/src/embed.rs:20 resize_bilinear`, `embed.rs:50 normalise`). ORT inference: session from model file, input [1,1,224,224], 384-d output checked against `EMBED_DIM` (`embed.rs:148 embed_onnx`, feature-gated). Fallback pseudo-embedding: chained SHA-256 of resized pixels expanded to a unit-norm 384-vector with WARN log whenever the model file is absent or the feature is off (`embed.rs:59 hash_embedding`, branch in `embed.rs:118 embed`). Cosine matching (`src/matcher.rs:11 cosine_similarity`); session pooling (`src/pool.rs EmbedderPool`); dl+mcc+bozorth fusion with weights fitted by EER minimisation (`src/fusion.rs:32 fuse`, `:44 fit_weights`, `:80 estimate_eer`).

**Model loading.** `DeepPrintEmbedder::from_env()` reads env var `EEL_FP_DL_ONNX_PATH` (`embed.rs:94`); `from_path()` takes an explicit path (`embed.rs:102`). Real inference additionally requires `--features onnx`.

**Public API.**

| Item | Signature |
|---|---|
| `DeepPrintEmbedder` | `from_env()` / `from_path(p)` / `preprocess(img)` / `embed(img) -> Result<Embedding, DlError>` |
| `cosine_similarity` | `fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32` |
| `DlMatcher` | `new(embedder)`; `compare(...)`; `compare_embeddings(a, b)` |
| `EmbedderPool` / `PooledEmbedder` / `fuse` / `fit_weights` / `EMBED_DIM = 384` | pooling and fusion utilities |

**Verification.** 16 tests (`tests/dl_test.rs`), all against the hash pseudo-embedding: dimension/unit-norm; identical embeddings cosine 1.0; different images below threshold; preprocess size; fusion bounds; weight-fit validity; pool serve/exhaust/recycle. No test exercises a real ONNX model.

---

## abis-fp-fpvte

**Purpose.** Evaluation harness replicating NIST FpVTE 2003 (NISTIR 7123) and PFT-III (NIST IR 8307): ROC/FMR/FNMR, rank-N identification, demographic fairness slicing, reports.

**Algorithms.** ROC sweep; EER interpolation (`services/abis-svc/crates/abis-fp-fpvte/src/harness.rs:269 compute_eer`); FMR@FNMR (`harness.rs:286`); FNMR/FMR at fixed threshold (`harness.rs:298`). Full run producing a Report with rank-N accuracy and fairness slices, JSON + CSV + text output (`harness.rs:41 FpVteHarness::run`). Synthetic corpus generator with unique subject IDs and optional demographic labels (`src/corpus.rs`). Pluggable `FpMatcher` trait with `NullMatcher` sentinel and deterministic `SyntheticMatcher` (first-64-byte agreement) (`src/matcher_trait.rs:16`).

**Public API.**

| Item | Signature |
|---|---|
| `FpVteHarness::run` | `fn run(&self, corpus: &Corpus, matcher: &dyn FpMatcher, output_dir: Option<&Path>) -> Result<Report, FpVteError>` |
| `HarnessConfig` | `roc_steps` (default 100), `max_rank` (default 10) |
| `Report` / `RocPoint` / `RankAccuracy` / `FairnessSlice` | report structures |
| `FpMatcher` | `name()`; `compare(probe, gallery: &Template) -> Result<f32, FpVteError>` |

**Verification.** 18 tests (`tests/fpvte_test.rs`): corpus counts and subject uniqueness; NullMatcher EER near 0.5; synthetic matcher self-scores 1.0; ROC/EER/rank-N/fairness ranges valid; JSON and CSV written; insufficient corpora rejected.

---

## abis-fp-sourceafis-bridge

**Purpose.** Pure-Rust reimplementation of the SourceAFIS algorithm (MPL-2.0 original; algorithm docs at sourceafis.machinezoo.com): skeletonisation, minutia extraction, ridge-structure descriptors, seed-and-extend matching.

**Algorithms.** Ridge-width-aware thinning that samples ridge width before thinning so short spurs suppress correctly (`services/abis-svc/crates/abis-fp-sourceafis-bridge/src/skeleton.rs:90 extract_skeleton`). Minutiae at skeleton end/branch points with local ridge direction (`src/minutia.rs:76 extract_minutiae`, `minutia.rs:32 ridge_direction`). Edge-table descriptor encoding relative position/direction plus geometrically estimated inter-minutia ridge counts (`src/ridge_structure.rs:51 build_edge_table`, `ridge_structure.rs:33 estimate_ridge_count`); rotation-invariant edge similarity (`ridge_structure.rs:97 edge_similarity`). Matching: highest-similarity seed pair, radial extension under seed alignment, score saturating near 40 paired minutiae via `raw/(raw+40)` (`src/matcher.rs:53 extend_from_seed`, `matcher.rs:138 score_minutiae`).

**Public API.**

| Item | Signature |
|---|---|
| `SourceAfisMatcher::compare` | `fn compare(&self, probe, gallery: &GrayImage) -> Result<f32, SourceAfisError>` |
| `SourceAfisMatcher::compare_minutiae` | `fn compare_minutiae(&self, probe, gallery: &[SourceAfisMinutia]) -> MatchScore` |
| Pipeline stages (exported) | `extract_skeleton`, `extract_minutiae`, `build_edge_table`, `edge_similarity` |
| `MatchScore` / `SourceAfisParams` | `paired`, `structure_score`, `ridge_score`, `combined()`; tunables |

**Verification.** 17 tests (`tests/sourceafis_test.rs`): skeleton dimensions and tiny-image rejection; stripes yield non-empty skeletons; one edge row per minutia without self-edges; descriptors rotation-invariant; identical sets score high, empty inputs zero; extension terminates on large sets.

---

## abis-enhancement (shared image crate)

**Purpose.** Classical image enhancement across modalities: fingerprint Gabor enhancement, slap segmentation, iris segmentation, face super-resolution.

**Algorithms.** Histogram equalisation (`services/abis-svc/crates/abis-enhancement/src/fingerprint.rs:38 histogram_equalize`); 15x15 Gabor filtering with DC-gain normalisation (`fingerprint.rs:72 gabor_filter`) averaged over 8 orientations (`fingerprint.rs:136 apply_gabor_bank`); thinning implemented as double 3x3 erosion, explicitly documented as simplified pending Hilditch/Zhang-Suen (`fingerprint.rs:156 morphological_thin`); restore pipeline equalise -> Gabor bank -> thin (`fingerprint.rs:194 restore`). Slap segmentation by even quarter-width strips (`src/slap.rs:38 segment_slap`) — simpler than `abis-fp-multifinger`'s valley approach. Iris: Daugman integro-differential operator, eyelid occlusion, segmentation (`src/iris.rs:41/:94/:120`). Face: bicubic upsample, unsharp mask, classical super-resolution (`src/face.rs:24/:58/:94`).

**Public API.** `histogram_equalize`, `gabor_filter`, `apply_gabor_bank`, `morphological_thin`, `restore`, `segment_slap(slap, hand) -> [SegmentedFinger; 4]`, `integro_differential_operator`, `detect_eyelid_occlusion`, `segment_iris`, `bicubic_upsample`, `unsharp_mask`, `super_resolve` — all operating on `GrayscaleImage`.

**Verification.** 15 tests (`tests/enhancement_tests.rs`, 160 lines), smoke-level over the pipelines on synthetic images.

---

## abis-rectification (shared workflow crate)

**Purpose.** Subject-driven biometric record-correction workflow (recapture, demographic update, template removal, duplicate merge) with 30-day SLA. Pure domain logic; no biometric algorithms.

**Mechanics.** State machine Submitted -> Triage -> EvidencePending -> Approved/Rejected -> Completed with invalid-transition errors (`services/abis-svc/crates/abis-rectification/src/workflow.rs:12` through `workflow.rs:88 complete`). SLA accounting: 30-day deadline, 3-day near-breach window (`src/domain.rs:68-69`, `domain.rs:91 sla_status`). Officer decision validation (`src/policy.rs PolicyEngine`); SLA sweep returning (near_breach, breached) counts (`src/scheduler.rs:8 sweep_sla`). `src/api.rs` contains only re-exports ("REST API stubs").

**Public API.** `RectificationRequest::new` (UUIDv7, SLA deadline set), `sla_status()`, transitions `to_triage` / `to_evidence_pending` / `assign_officer` / `decide` / `complete`, `PolicyEngine::validate_decision`, `sweep_sla(requests) -> (usize, usize)`.

**Verification.** 14 tests (`tests/rectification_tests.rs`): legal/illegal transitions, officer assignment rules, decision validation, SLA boundaries, sweep counting.

---

## Notes, caveats, known gaps

Stubs and deferrals:

- `abis-rectification/src/api.rs` is an explicit stub; no HTTP layer exists, and the "SLA sweep every 15 minutes" cadence mentioned in crate docs is not implemented.
- `abis-enhancement/src/fingerprint.rs:156 morphological_thin` is a placeholder per its own doc comment; its slap segmentation is naive next to `abis-fp-multifinger`.
- `abis-fp-fpvte`'s `SyntheticMatcher` compares first 64 template bytes — harness scaffolding, not a matcher.
- `abis-fp-multifinger`'s matcher compares fingers via intensity-histogram Bhattacharyya coefficient (`src/matcher.rs:24`), not minutiae; scores are a proxy until a real per-finger matcher is wired in.

Model/artefact dependencies:

- `abis-fp-dl-onnx`: real inference needs the `onnx` cargo feature plus an `.onnx` file located via `EEL_FP_DL_ONNX_PATH` or `from_path`. Without either, embeddings come from the SHA-256 pseudo-embedding, which carries no biometric information — scores produced in this mode must not be treated as genuine match scores. Further, `embed_onnx` uses `ndarray::Array4` (`embed.rs:166`) but `ndarray` is absent from the crate's `Cargo.toml`, so the `onnx` feature path appears not to compile as-is and is untested.
- `abis-fp-nfiq22`: the compiled-in forest is a minimal synthetic default (`DEFAULT_MODEL` in `model.rs`); production requires loading the NIST-published model via `RandomForestModel::from_path`. Test claims hold only against synthetic images and the synthetic model.

Thin test coverage:

- `abis-enhancement` (15 tests / 547 src lines) and `abis-rectification` (14 / 324) are lightest relative to surface area, mostly smoke tests.
- `abis-fp-nfiq22` has the fewest fp-crate tests (13) and none exercise a calibrated production model.
- `abis-fp-dl-onnx` never touches the ORT path (CI has no model artefact); pooling and weight fitting are covered only against pseudo-data.

Other observations:

- Five-plus near-identical `GrayImage` definitions across these crates would benefit from consolidation into a shared type.
- Only `abis-fp-consensus` (and downstream `abis-fp-vendor-peak`) integrate the individual matchers; the rest are unreachable from any service binary in this workspace.
- Angle-unit inconsistency: MCC converts minutia theta degrees -> radians (`abis-fp-mcc/src/build.rs:64`), i.e. `FingerprintTemplate.minutiae[].theta` is degrees, while `abis-fp-tps-warp`'s `Minutia.theta` is radians (`lib.rs:39`). Consumers bridging them must convert.
