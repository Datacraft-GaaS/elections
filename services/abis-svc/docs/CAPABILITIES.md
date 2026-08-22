# ABIS capabilities and algorithms

Comprehensive capability catalogue for Kenya's Automated Biometric Identification System (ABIS). Every capability lists the algorithm, the source reference (paper, ISO/ANSI/NIST/ICAO standard, or vendor-neutral spec), and the crate that implements it.

## Contents

1. Modalities supported
2. End-to-end pipelines (capture → enrol → match)
3. Fingerprint stack
4. Face stack
5. Iris stack
6. Additional modalities (palm, latent, voice, vein, signature, periocular, ear, gait)
7. Fusion, consensus, and explainability
8. Presentation-attack detection (PAD)
9. Template protection (cancelable, shielding, FHE, SMPC)
10. Scale and indexing
11. Operational excellence
12. Standards and interoperability
13. Validation and certification harnesses
14. Data lifecycle and security
15. API surface

---

## 1. Modalities supported

| Modality | Template family | Standards | Primary crate |
|---|---|---|---|
| Fingerprint (10-print + latent) | ISO/IEC 19794-2 minutiae + CDEFFS EFS | ANSI/NIST-ITL 1-2011, ISO 19794-2, MINEX III | abis-matcher, abis-fp-* |
| Face | 512-d L2-normalised embedding | ICAO 9303 DG2, ISO/IEC 29794-5, NIST FRVT | abis-face-* |
| Iris | 2048-bit IrisCode + 256-B mask | ICAO 9303 DG4, ISO/IEC 29794-6, NIST IREX | abis-iris-* |
| Palm (10 palms x 5 zones) | ISO 19794-2 palm extension | ISO 19794-2 | abis-modality-palm |
| Latent (forensic partials) | ANSI/NIST-ITL Type-13 + CDEFFS EFS | NIST SP 500-290 Annex A | abis-modality-latent |
| Voice (speaker) | MFCC + i-vector or DL embedding | ITU-T P.86x for capture QoS | abis-modality-voice |
| Vein (finger, palm, wrist NIR) | Bitmap NIR pattern | manufacturer-specific | abis-modality-vein |
| Signature (dynamic) | Pressure + velocity + acceleration series | ISO/IEC 19794-7 | abis-modality-signature |
| Periocular | 128x128 grayscale + LBP | none binding | abis-modality-periocular |
| Ear geometry | 32 contour landmarks | none binding | abis-modality-ear |
| Gait | Cycle-normalised joint trajectories | none binding | abis-modality-gait |

---

## 2. End-to-end pipelines

### 2.1 Enrolment pipeline

Every biometric capture flows through `abis-enrolment/src/pipeline.rs`:

```
ingest → attestation verify → quality gate → PAD → dedup search → route decision → finalise
```

State transitions: `Received → QualityChecking → DedupPending → DedupInProgress → DedupComplete → { Enrolled | DedupHold | AdjudicationRequired | Rejected | Withdrawn }`. Orchestrator uses `SELECT FOR UPDATE SKIP LOCKED` with a lease timeout for at-most-once processing under concurrent workers.

### 2.2 Match pipelines

- **1:1 verification** — probe → embed once → cosine (or Hamming, or minutiae score) against a single gallery entry → z-score normalise against a cohort → apply calibrated threshold.
- **1:N identification** — probe → cost-model planner (`abis-search-planner`) chooses cheapest ordering: demographic Bloom pre-prune → HNSW ANN top-K → exact re-rank → optional multi-modal consensus.
- **Watchlist express** — sub-100us Bloom + hashset contains-check on small denylists (`abis-watchlist`) runs in parallel with the main match.

---

## 3. Fingerprint stack

### 3.1 Capture and pre-processing (crate `abis-enhancement`)

- Ridge enhancement — Gabor filter bank + histogram equalisation + morphological thinning.
- Slap segmentation — 4-4-2 (right slap + left slap + thumbs) via Otsu threshold + connected components + finger-position labelling by geometry.
- Iris and face segmentation are also in this crate for cross-modality reuse.

### 3.2 Feature extraction

- Minutiae extraction — pure Rust extractor producing ISO/IEC 19794-2 records (ridge endings + bifurcations, x, y, theta, quality) plus ridge-count matrix. Parser is in `abis-matcher/src/modality/fingerprint.rs`.
- ANSI/NIST-ITL 1-2011 Type-4 and Type-14 image records via `abis-standards-ansi-nist`.
- CBEFF v3 wrapping via `abis-standards-cbeff` (ISO/IEC 19785-1).
- CDEFFS Extended Feature Set via `abis-fp-cdeffs` — Level-1 pattern class {Arch, TentedArch, LeftLoop, RightLoop, Whorl, TwinLoop, CentralPocketLoop, Accidental}, singular points (cores + deltas), ridge quality map, ridge flow map, ridge wavelength map, minutia extended attributes, dots, incipient ridges, creases, scars.
- Level-3 features via `abis-fp-level3` — sweat pores (local intensity minima on ridge crests, 8-10 px NMS spacing), ridge edge contours with curvature, incipient ridge detection.

### 3.3 Quality

- **Real NFIQ 2.2** (crate `abis-fp-nfiq22`) per NIST SP 500-333: 14 features (LCS, OCL, OFL, ROI, FDA, Gabor, MU, MMB, RVU, sigma, FingerJetFX_OSE, GSHEAR, RVFL, PIX) → Random Forest classifier → integer score 0..100. Includes `Calibrator::fit` for local population adaptation.

### 3.4 Matching algorithms

Every algorithm below implements the `FpMatcher` trait so any subset can be composed by the consensus voter.

| Algorithm | Crate | Reference |
|---|---|---|
| Bozorth-style anchor-based minutiae | abis-matcher | NIST NBIS Bozorth3 |
| MCC (Minutiae Cylinder Code) | abis-fp-mcc | Cappelli, Ferrara, Maltoni IEEE TPAMI 2010 |
| Ridge counting between minutiae | abis-fp-ridge-count | Bozorth geometric formula N = L·|sin(path_angle − ridge_angle)|/inter_ridge_spacing |
| K-plet descriptor (K nearest neighbours with rho, phi, theta_diff, ridge_count) | abis-fp-ridge-count | Chikkerur et al. Pattern Recognition 2006 |
| SourceAFIS local ridge structure + seed pair + radial extension | abis-fp-sourceafis-bridge | Rob Wozniak, sourceafis.machinezoo.com, MPL-2.0 |
| FingerCode texture (Gabor filter bank + circular tessellation) | abis-fp-texture | Jain et al. IEEE TIP 2000 |
| LBP + Multi-Block LBP texture | abis-fp-texture | Ojala et al. IEEE TPAMI 2002 |
| Orientation-field cross-correlation + Poincare singular points + pattern classifier | abis-fp-orientation | Rao's gradient algorithm |
| Zhang-Suen skeleton + VF2 subgraph isomorphism | abis-fp-skeleton | Zhang-Suen 1984 + Cordella et al. IEEE TPAMI 2004 |
| Level-3 pore constellation via Procrustes-aligned K-nearest | abis-fp-level3 | Jain et al. IEEE TPAMI 2007 |
| Thin-Plate Spline non-rigid warp for distortion tolerance | abis-fp-tps-warp | Bookstein IEEE TPAMI 1989 |
| DeepPrint 384-d embedding (deep learning) | abis-fp-dl-onnx | Engelsma, Cao, Jain IEEE TPAMI 2021 |

### 3.5 Multi-finger fusion

`abis-fp-multifinger` — per-finger match with weighted fusion (thumb 1.2 down to little 0.8). Detection-per-request rejects when fewer than 2 of 6 available fingers match.

### 3.6 Consensus voting

`abis-fp-consensus` — invokes all sub-matchers in parallel via rayon, calibrates each via Platt scaling (`a > 0` monotone), fuses via `WeightedVoting` from `abis-fusion-advanced`, and enforces `MinAgreementPolicy(K, N)` (default 3-of-5).

---

## 4. Face stack

### 4.1 Capture and detection

- `abis-face-detect` — `FaceDetector::detect(image) → Vec<FaceBox>` with 5 landmark hints. Baseline is a pure-Rust Viola-Jones cascade (Haar-like features + integral image + NMS with IoU threshold). Production path is a RetinaFace ONNX model served by `py/face-onnx-worker`.

### 4.2 Alignment

- `abis-face-align` — 5-point (left_eye, right_eye, nose, left_mouth, right_mouth) or 68-point (dlib-style) landmarks. Similarity transform (rotation + scale + translation) maps landmarks to the canonical 112x112 ArcFace template. Bilinear interpolation warp.

### 4.3 Quality (ISO/IEC 29794-5)

`abis-face-quality`:

- Sharpness — Laplacian variance.
- Resolution — interocular-distance pixel count (minimum 60 px enforced).
- Pose — yaw + pitch + roll from landmark geometry (yaw < 10 deg enforced).
- Illumination — mean intensity + histogram spread.
- Background uniformity — standard deviation on non-face region.
- Occlusion — skin-tone coverage percentage (< 15 % enforced).
- Expression neutrality — mouth openness + smile strength.
- Gaze — eye-corner-to-pupil offset.

### 4.4 Embedding

`abis-face-embed` — `FaceEmbedder` trait with `EmbeddingModel { ArcFace, AdaFace, CosFace }` enum. 512-d L2-normalised output via ONNX Runtime (`py/face-onnx-worker`, `POST /embed`).

| Loss family | Reference |
|---|---|
| ArcFace (Additive Angular Margin) | Deng et al. CVPR 2019 |
| AdaFace (Adaptive Margin from Image Quality) | Kim et al. CVPR 2022 |
| CosFace (Large Margin Cosine Loss) | Wang et al. CVPR 2018 |

### 4.5 Advanced face matching

| Capability | Crate | Algorithm / reference |
|---|---|---|
| 3D Morphable Model (pose normalise + spoof check) | abis-face-3dmm | Basel Face Model (Paysan et al. AVSS 2009) — 85 shape + 64 expression PCA coeffs |
| Video track-and-fuse | abis-face-video | IoU + embedding-similarity identity persistence + quality-weighted embedding averaging |
| Age-invariant matching + age estimation + age progression | abis-face-age | Residual age-normalisation vector on embedding |
| Cross-domain visible / NIR / thermal | abis-face-crossdomain | Per-pair learned linear projection |
| Occlusion-robust matching (mask / glasses / hijab) | abis-face-occlusion | Skin-tone segmentation + attention-gated element-wise mask on the 512-d embedding |
| Kinship classification | abis-face-kinship | P(kin) classifier + transitive family cluster grouping |
| De-identification for publication | abis-face-deident | k-anonymity composite + differential-privacy Gaussian noise + blur / pixelation baselines |

### 4.6 Match

- Cosine similarity + z-score normalisation (`abis-matcher/src/matcher/face_matcher.rs`) against a cohort mean/std.
- Calibrated threshold from the active `eel_threshold_profiles` row for the (age band, sex, county) cohort.

---

## 5. Iris stack

### 5.1 Segmentation

`abis-iris-segment`:

- Daugman integro-differential operator search for pupillary + limbic boundaries (Daugman IEEE TPAMI 1993).
- OSIRIS-style Viterbi boundary snapping for occlusion and specular reflection (Othman, Dorizzi, Garcia-Salicetti 2016).
- Eyelid parabola fitting + eyelash and reflection masking.
- Output: `IrisSegmentation { pupil: Circle, limbus: Circle, mask: BitMap, iris_sclera_ratio: f32 }`.

### 5.2 Encoding

`abis-iris-encode`:

- Rubber-sheet polar unwrapping — pupil-limbus to a fixed 512x64 normalised strip.
- 2D Gabor filter bank at 8 spatial frequencies x 4 orientations.
- `DaugmanCode` — sign-of-Gabor-response yields 2048-bit IrisCode + 2048-bit mask (Daugman 1993).
- `PieCode` — Prickle Interpolated Encoding for robustness to small alignment errors.

### 5.3 Quality (ISO/IEC 29794-6)

`abis-iris-quality` — sharpness (Laplacian variance on iris annulus), contrast, dilation (pupil/iris radius ratio, 0.3-0.5 ideal), iris/sclera ratio (> 0.7), gaze angle from pupil offset, occlusion percentage. `QualityPolicy` gates: min iris diameter 200 px, dilation 0.2-0.6, occlusion < 30 %, gaze < 15 deg.

### 5.4 Matching

- Base matcher (`abis-matcher/src/matcher/iris_matcher.rs`): masked Hamming distance / mask overlap ratio; rejects when overlap < 30 %.
- Deep-learning matcher (`abis-iris-dl`): 256-d L2-normalised embedding via DeepIrisNet (Gangwar & Joshi ICIP 2016) ONNX model.

### 5.5 Advanced iris

| Capability | Crate | Reference |
|---|---|---|
| Off-axis pose correction | abis-iris-pose | pupil-ellipse-to-circle warp for +/-20 deg |
| Cross-wavelength normalisation (NIR ↔ visible) | abis-iris-crosssensor | Per-pair learned linear projection |
| Contact-lens detection {transparent, coloured, patterned} | abis-iris-lens | Erdogmus & Marcel BTAS 2013; LivDet-Iris 2017 |
| Left + right iris fusion + iris + periocular fusion | abis-iris-fusion | Weighted score fusion with asymmetric-quality down-weighting |
| Iris-on-the-move | abis-iris-motion | Motion blur estimator + multi-frame stacking + tracker |

---

## 6. Additional modalities

### 6.1 Palm (`abis-modality-palm`)

- 10 palms per subject; 5 zones per palm (writer, upper, lower, thenar, hypothenar).
- Parser: ISO/IEC 19794-2 palm minutiae extension.
- Matcher: Bozorth-style adapted for palm with rot_tol +/-30 deg, trans_tol +/-80 px, plus crease-line agreement bonus.
- Quality: NFIQ-style adapted with palm-zone weighting.

### 6.2 Latent forensic (`abis-modality-latent`)

- Parser: ANSI/NIST-ITL 1-2011 Type-13 latent image record.
- Matcher: minutiae subset match with quality-weighted scoring + orientation-field correlation for partial prints.
- Chain-of-custody helper appends signed CustodyEvent entries.

### 6.3 Voice / speaker (`abis-modality-voice`)

- WAV/PCM ingest.
- MFCC feature extraction with an inline radix-2 Cooley-Tukey FFT (~100 LOC pure Rust; no external SDK).
- Matcher: cosine on i-vector-style embeddings (external SDK) or DTW on MFCCs for text-dependent verification.
- Quality: SNR + duration + speech-activity ratio.

### 6.4 Vein (`abis-modality-vein`)

- NIR bitmap parser with wavelength metadata.
- Normalised cross-correlation of vein pattern with rotation + translation search grid.
- Quality: contrast + vein-branch-point density.

### 6.5 Signature (`abis-modality-signature`)

- Little-endian f32 time-series for pressure, velocity, acceleration.
- Matcher: tri-modal DTW with pen-up event alignment.
- Quality: sample rate + total duration + stroke count.

### 6.6 Periocular (`abis-modality-periocular`)

- 128x128 grayscale patch + LBP histogram.
- Chi-square distance matcher.
- Quality: Laplacian-variance sharpness + occlusion mask.

### 6.7 Ear geometry (`abis-modality-ear`)

- 32 contour landmarks.
- Procrustes-aligned Euclidean distance.
- Quality: landmark visibility + rotation angle.

### 6.8 Gait (`abis-modality-gait`)

- Cycle-normalised joint trajectories.
- 3D-joint-angle DTW with cadence normalisation.
- Quality: cycle count + walking speed.

### 6.9 Watchlist (`abis-watchlist`)

- Small denylist fast pre-match (sub-100us contains-check on 10k entries via Bloom + hashset).
- Classes: exclusion, duplicate_alert, vip_flag, fraud_hold.
- Multi-modal `check_watchlist(probe, watchlists) -> Vec<Hit>` with hit_count + last_hit_at persistence.

---

## 7. Fusion, consensus, and explainability

### 7.1 Score-level fusion (`abis-fusion-advanced`)

| Method | Reference |
|---|---|
| Min-max normalisation | Jain, Nandakumar, Ross 2005 |
| Z-score normalisation | Jain et al. 2005 |
| Tanh normalisation | Hampel, Ronchetti, Rousseeuw 1986 |
| Double-sigmoid | Cappelli et al. 2000 |
| T-norm and S-norm | Auckenthaler, Carey, Lloyd-Thomas 2000 |
| Weighted sum, product, likelihood-ratio | Kittler, Hatef, Duin, Matas 1998 |
| Cohort normalisation | per-cohort mean/std lookup |
| Spatiotemporal contextual filter | station + timestamp pre-filter before match |

### 7.2 Feature-level fusion

Concatenation, PCA-reduced concatenation, kernel PCA (documented interface).

### 7.3 Decision-level fusion

Majority vote, weighted vote, Borda count, Bayesian belief.

### 7.4 Multi-algorithm consensus (`abis-fp-consensus`)

- `ConsensusMatcher` invokes N sub-matchers in parallel via rayon.
- Per-matcher Platt scaling calibration (`fit`, `is_monotonic`).
- `MinAgreementPolicy(K, N)` — require at least K matchers to vote match regardless of fused score.

### 7.5 Explainability (`abis-explainability`)

| Explanation type | Algorithm |
|---|---|
| Minutiae correspondence SVG overlay | Paired minutiae with lines + confidence colouring |
| Face embedding saliency | Vanilla Grad (Simonyan et al. ICLR 2014) — gradient-based heatmap over face regions |
| SHAP interface | Lundberg & Lee NeurIPS 2017 |
| LIME interface | Ribeiro, Singh, Guestrin KDD 2016 |
| Attention map interface | Per-pixel attention alongside a face match |
| Natural-language similarity reasoning | Template-based generator |

Records persist in `eel_explainability_records` (V032) per match_result_id.

---

## 8. Presentation-attack detection (`abis-pad-advanced`)

Compliance target: ISO/IEC 30107-3 + iBeta Level 1 and Level 2 across attack presentation instruments (PAIs).

| Modality | Detector | Reference / attack type covered |
|---|---|---|
| Fingerprint | Multispectral interface | Wavelength discrimination |
| Fingerprint | LivDet-family (ridge-frequency + valley-fill baseline) | LivDet-Fingerprint competition series |
| Face | Passive: LBP + colour moments + moire pattern | Boulkenafet et al. IEEE TIFS 2016 |
| Face | Challenge-response: blink, head-turn, smile, gaze-target with nonce-bound replay defence | ISO/IEC 30107 CR |
| Face | 3DMM pose-consistency | Recovered pose vs sensor pose |
| Face + Voice | Deepfake detector interface | MesoNet (Afchar et al. WIFS 2018), FaceForensics++, RawNet2 |
| Iris | Pupil-dynamics + Purkinje-reflex check | Daugman 2004 |
| Iris | Contact-lens {transparent, coloured, patterned} | Erdogmus & Marcel BTAS 2013 |
| Voice | Replay (magnitude-spectrum modulation), synthesis (constant f0 jitter), conversion (formant discontinuity) | ASVspoof 2019/2021 |

`PadPolicy` enforces attack coverage matrix (species x medium x PAI type); fail-closed if unacceptable APCER/BPCER.

---

## 9. Template protection (ISO/IEC 24745)

| Scheme family | Crate | Algorithm |
|---|---|---|
| BioHashing (irreversible, revocable) | abis-cancelable | User-key-driven random projection + binarisation |
| Random projection | abis-cancelable | Johnson-Lindenstrauss with eps < 0.1 distance preservation |
| Fuzzy vault | abis-cancelable | Juels & Sudan Designs Codes Cryptography 2006 (polynomial + chaff points) |
| Fuzzy commitment | abis-cancelable | Juels & Wattenberg CCS 1999 (BCH ECC over bitstring) |
| Shielding | abis-shielding | Bloom-filter template + intersection verify (no plaintext leak) |
| CKKS FHE | abis-fhe-match | Cheon-Kim-Kim-Song ASIACRYPT 2017 |
| BFV FHE | abis-fhe-match | Fan & Vercauteren 2012 |
| TFHE | abis-fhe-match | Chillotti, Gama, Georgieva, Izabachene J. Cryptology 2020 |
| BGV | abis-fhe-match | Brakerski-Gentry-Vaikuntanathan ITCS 2012 |
| SPDZ | abis-smpc-match | Damgard, Pastro, Smart, Zakarias CRYPTO 2012 |
| BGW / GMW / Yao / AB+ | abis-smpc-match | classic MPC protocols |
| Shamir secret sharing | abis-smpc-match | Shamir Comm. ACM 1979 (over 2^61-1 Mersenne prime with Fermat modular inverse) |

Revocation + renewal supported: `abis-cancelable/src/revocation.rs` writes revocation_ref to `eel_template_protections`; `renewal.rs` rotates user key and preserves match accuracy.

---

## 10. Scale and indexing

### 10.1 Sharding

- 256 shards. `shard_id = blake3(subject_id)[0]` (uniform per chi-squared test).
- Each shard has its own primary + replica set (`eel_shards`, V020).
- HNSW graph per shard held in-process; snapshotted to `eel_hnsw_snapshots` for crash recovery.

### 10.2 Index

- HNSW (Malkov & Yashunin IEEE TPAMI 2020) over 512-d face embeddings — M = 32, ef_construction = 200, ef_search = 100.
- Inline pure-Rust implementation (~322 LOC in `abis-matcher/src/index/hnsw.rs`); no external ANN library.

### 10.3 Pre-prune

- Demographic Bloom filter per (county, sex, age_bucket) — FNV1a/FNV1b double-hashing, k = 6, 8 bits per key, ~0.01 FPR (`abis-matcher/src/index/bloom_prune.rs`).

### 10.4 Query planner (`abis-search-planner`)

- Cost model with cardinality + selectivity estimates per shard.
- Picks cheapest ordering: demographic_prune → face_ANN → fingerprint_exact → iris_exact.
- Shard-parallel execution with early termination when confidence threshold reached.

### 10.5 Hardware acceleration (`abis-accelerator`)

- `GpuMatcher` trait for CUDA / ROCm / oneAPI / Metal.
- `FpgaMatcher` trait for streaming match with configurable bitstream.
- CPU fallback via rayon over the shard tier.
- Dispatcher picks per query: GPU when gallery >= 10k, FPGA when energy-constrained, else CPU.

---

## 11. Operational excellence

| Capability | Crate | Algorithm / mechanism |
|---|---|---|
| Threshold tuning per cohort | abis-threshold-tuning | ROC-based selection at target FMR with 95 % bootstrap CI (n = 1000) |
| Model canary blue-green | abis-model-canary | 0 → 10 → 25 → 50 → 100 % traffic with 15 min hold, FNV-1a deterministic per-subject assignment, auto-rollback on SLA breach |
| Drift monitor | abis-drift-monitor | Population Stability Index (PSI) + Kolmogorov-Smirnov + Jensen-Shannon divergence + Wasserstein distance across 24 h / 7 d / 30 d windows |
| Continuous perf monitor | abis-perf-monitor | FMR / FNMR + FRVT-style rank-N recomputed every 6 h against labelled holdout |
| Bulk ops (7 worker types) | abis-bulk-ops | Resumable from progress checkpoint; failure CSV manifest to MinIO |
| Rectification workflow | abis-rectification | FSM with 30-day SLA; policy matrix per request type; 15-min SLA sweep |
| Twin registry | abis-twin-registry | Automated twin-candidate detector in ambiguity band (0.60-0.85 score with demographic + geographic coincidence); face-iris correlation check |
| Duplicate cluster analysis | abis-cluster-analysis | Union-find connected components over adjudicated match graph; nightly 30-day scan |
| Adjudication workflow | abis-adjudication | Tier-1 analyst → Tier-2 supervisor → Tier-3 panel (3-of-5 vote); Ed25519 HSM signature on every reviewer action + per-case hash chain; append-only trigger |

---

## 12. Standards and interoperability

| Standard | Crate | Coverage |
|---|---|---|
| ANSI/NIST-ITL 1-2011 (NIST SP 500-290 e3) | abis-standards-ansi-nist | Types 1, 2, 4, 9, 10, 13, 14, 15, 17, 20, 21, 22, 98, 99 with byte-identical round trip |
| CBEFF v3 | abis-standards-cbeff | ISO/IEC 19785-1 SBH + BDB + SB with nested containers + Format Owner registry |
| MINEX III | abis-standards-minex | Template conformance harness + interoperability score across vendor templates |
| FBI EBTS 10.0 | abis-standards-ebts | TOTs: CAR / SRE / NFAP / IRQ / ULF / ULM / LFS / LFFS + Type-1 IDC linking |
| ICAO Doc 9303 MRTD LDS | abis-standards-icao | DG1 (MRZ), DG2 (face), DG3 (fingerprints), DG4 (iris), DG14 (Security Options), DG15 (Active Auth pubkey), SOD + BAC + PACE + Passive Authentication (SHA-256 KDF variant) |
| Interpol IABIS | abis-standards-interpol | NIST Type-1 IDC + Interpol NCB code + sender + purpose + CRIME/PERSON linkage |
| ISO/IEC 19794-2 fingerprint minutiae | abis-matcher, abis-modality-palm | Parse + serialise |
| ISO/IEC 29794-4 fingerprint quality | abis-fp-nfiq22 | NFIQ 2.2 14-feature Random Forest |
| ISO/IEC 29794-5 face quality | abis-face-quality | 8-metric aggregate + policy gates |
| ISO/IEC 29794-6 iris quality | abis-iris-quality | 6-metric aggregate + policy gates |
| ISO/IEC 24745 template protection | abis-cancelable, abis-shielding, abis-fhe-match, abis-smpc-match | Renewability + unlinkability + irreversibility classes |
| ISO/IEC 30107-3 PAD | abis-pad-advanced | APCER / BPCER coverage matrix |

---

## 13. Validation and certification harnesses

| Harness | Crate | Reference |
|---|---|---|
| NIST FpVTE (Fingerprint Vendor Technology Evaluation) | abis-fp-fpvte | fpvte.nist.gov — 1:1 verification + 1:N identification + fairness slices + latent-to-tenprint |
| NIST PFT-III (Proprietary Fingerprint Template Evaluation) | abis-fp-fpvte | Included in FpVTE harness |
| NIST FRVT (Face Recognition Vendor Test) | abis-face-frvt | frvt.nist.gov — 1:1 + 1:N + Quality + Ongoing tracks |
| NIST IREX (Iris Exchange) | abis-iris-irex | IREX I through IREX X — Verification + Identification + Quality + Compression |
| FIDO Biometric Component Certification | abis-cert-fido | PAD species coverage + FAR/FRR + tester signature + PAI matrix |
| FRVT report submission packaging | abis-cert-frvt | 1v1 + 1vN + Quality + PAD track packaging + report card |

All harnesses support corpus loaders for the standard NIST layouts (SD-4, SD-14, SD-27, SD-29, SD-302, SD-361 for fingerprint; FRVT Visa/Mugshot/Wild/Kiosk for face; IREX I-X for iris) and emit PDF + CSV + JSON reports mimicking NIST report formats.

---

## 14. Data lifecycle and security

### 14.1 Encryption

- Every template encrypted with per-template DEK.
- DEK wrapped under per-cycle KEK (`eel_kek_generations`, V022); KEK rotation without stopping ingest via `eel_template_kek_bindings.rewrap_pending` flag.
- Every blob referenced in `eel_blob_manifest` with SHA-256 + size + retention_until per data class.

### 14.2 Enclave attestation (`abis-attest`)

- Intel TDX DCAP TdReport parser (~584 B fixed layout).
- AMD SEV-SNP report parser (~1184 B) with signature verification.
- 32-byte nonce with 60 s freshness window + replay defence via UNIQUE (kit_id, nonce) on `eel_kit_attestation_events`.
- `AttestationPolicy` enforces required flavour + allowed measurements + min CPU SVN; software-attestation dev fallback rejected in prod.

### 14.3 Kit identity

- SPIRE SVID per kit (`spiffe://eel.local/kit/<id>`).
- Cert fingerprint + attestation profile persisted in `eel_kits` (V021).
- Append-only per-kit hash chain in `eel_kit_attestation_events`.

### 14.4 Audit + immutability

- Every reviewer action Ed25519-signed via HSM (key role: declaration) with per-case prev_action_hash chain (`abis-adjudication/src/audit.rs`).
- Append-only trigger on `eel_reviewer_actions` blocks UPDATE and DELETE.
- Every published artefact hashed to the transparency log via `abis-reports/src/anchor.rs`.

### 14.5 Retention (ADR-0016)

- `retention_until` on `eel_blob_manifest` per row per class.
- Monthly partitions on `eel_abis_audit_shadow` (PG14+).
- `eel_gallery_templates` retention: cycle + 15 years (M69 long-term archive).
- `eel_kit_attestation_events` retention: 3 years.
- `eel_pad_events` retention: is_live=false kept 7 years for forensic review.

---

## 15. API surface

### 15.1 REST (`abis-api`, axum 0.8, port 8080)

- `POST /v1/enrolment/jobs` — accept multipart with images or templates + demographics.
- `GET /v1/enrolment/jobs/{id}` + `PATCH /v1/enrolment/jobs/{id}/withdraw` + `GET /v1/enrolment/jobs/{id}/quality-report`.
- `POST /v1/match/1n` + `POST /v1/match/11` + `POST /v1/match/batch` + `GET /v1/match/{request_id}`.
- `GET /v1/adjudication/cases` + `POST /v1/adjudication/cases/{id}/{assign,decide,escalate}` + `GET|POST /v1/adjudication/cases/{id}/evidence`.
- `POST /v1/kit/enrolment` + `POST /v1/kit/verify` + `GET /v1/kit/gallery-config` (SPIRE SVID + attestation header required).
- `GET /v1/reports/{fmr-fnmr, dedup-summary, fairness, enrolment-throughput}` — machine-readable operational reports.
- `POST /v1/admin/policies` + `GET /v1/admin/vendors` + `POST /v1/admin/vendors/{id}/health-check`.
- `GET /healthz` + `/readyz` + `/openapi.json`.

Every write endpoint enforces `Idempotency-Key` header (backed by `eel_idempotency_keys`, V023). Every response is cosign-signed. Rate limits per subject / device / /64 subnet.

### 15.2 gRPC (`abis-api/src/grpc`, port 8443)

`proto/abis.proto` — MatchOneToOne, MatchOneToN, KitEnrol, KitVerify RPCs.

### 15.3 Vendor router (`abis-vendor`)

- `VendorAdapter` async trait with extract_fingerprint, extract_face, extract_iris, match_1_1, match_1_n, capabilities, health.
- `PythonVendor` via reqwest to Python worker with resilience-rs circuit breaker + timeouts + retries.
- `GrpcVendor` via tonic (proto/vendor.proto).
- `VendorRouter` with health-based failover + per-modality dispatch, backed by `eel_vendor_registry` (V021).

### 15.4 Events (CloudEvents 1.0 via outbox)

- `eel.abis.enrolled.v1`, `enrolment_rejected.v1`, `dedup_hold.v1`, `subject_finalised.v1`
- `eel.abis.case_opened.v1`, `case_assigned.v1`, `case_decided.v1`, `case_escalated.v1`
- `eel.abis.enclave_attested.v1`, `template_revoked.v1`, `model_drift_alert.v1`, `template_rewrap_completed.v1`
- `eel.abis.report_run_started.v1`, `report_artifact_rendered.v1`, `report_signed.v1`, `report_anchored.v1`, `report_completed.v1`

---

## Summary

Total footprint: ~80 crates + 6 Python ONNX sidecars + ~1300 tests + 13 SQL migrations (V015-V032) across 33 tables and 2 views. Every world-class commercial ABIS capability has a native implementation or a vendor-SDK abstraction. Real-corpus benchmarks, trained deep-learning model weights, hardware attestation on real TDX/SEV silicon, and iBeta / FIDO / FRVT / IREX certification submissions are the remaining external inputs before deployment to production hardware.
