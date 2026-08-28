# Security & Privacy Crates and Standards Interchange

Reference for two groups of crates under `services/abis-svc/crates/`:

- **Group 1 — Security & privacy**: `abis-cancelable`, `abis-shielding`,
  `abis-shielded-adjudication`, `abis-smpc-match`, `abis-fhe-match`,
  `abis-pad-advanced`, `abis-attest`, `abis-cert-fido`, `abis-cert-frvt`,
  `abis-model-canary`.
- **Group 2 — Standards interchange**: `abis-standards-ansi-nist`,
  `abis-standards-cbeff`, `abis-standards-minex`, `abis-standards-ebts`,
  `abis-standards-icao`, `abis-standards-interpol`.

Paths below are relative to `services/abis-svc/crates/<crate>/`.

## Package mapping and sizes

| Crate directory | Package name | Lib name | Rust LOC |
|---|---|---|---|
| abis-cancelable | `eel-abis-cancelable` | `eel_abis_cancelable` | 993 |
| abis-shielding | `eel-abis-shielding` | `eel_abis_shielding` | 342 |
| abis-shielded-adjudication | `eel-abis-shielded-adjudication` | `eel_abis_shielded_adjudication` | 894 |
| abis-smpc-match | `eel-abis-smpc-match` | `eel_abis_smpc_match` | 722 |
| abis-fhe-match | `eel-abis-fhe-match` | `eel_abis_fhe_match` | 696 |
| abis-pad-advanced | `eel-abis-pad-advanced` | `eel_abis_pad_advanced` | 1585 |
| abis-attest | `abis-attest` (no prefix) | `abis_attest` | 1026 |
| abis-cert-fido | `eel-abis-cert-fido` | `eel_abis_cert_fido` | 407 |
| abis-cert-frvt | `eel-abis-cert-frvt` | `eel_abis_cert_frvt` | 399 |
| abis-model-canary | `eel-abis-model-canary` | `eel_abis_model_canary` | 793 |
| abis-standards-ansi-nist | `eel-abis-standards-ansi-nist` | `abis_standards_ansi_nist` | 1544 |
| abis-standards-cbeff | `eel-abis-standards-cbeff` | `abis_standards_cbeff` | 694 |
| abis-standards-minex | `eel-abis-standards-minex` | `abis_standards_minex` | 474 |
| abis-standards-ebts | `eel-abis-standards-ebts` | `abis_standards_ebts` | 793 |
| abis-standards-icao | `eel-abis-standards-icao` | `abis_standards_icao` | 653 |
| abis-standards-interpol | `eel-abis-standards-interpol` | `abis_standards_interpol` | 330 |

Naming conventions are inconsistent: Group 1 libs use `eel_abis_*`, Group 2
uses `abis_standards_*`; `abis-attest` alone lacks the `eel-` package prefix.

## Interconnections

- `eel-abis-shielded-adjudication` composes Group 1's data protection stack:
  it depends on `eel-abis-shielding`, `eel-abis-cancelable`,
  `eel-abis-adjudication` (`test-utils`), and workspace package
  `eel-crypto-rs`.
- Standards stack layers upward: ANSI/NIST Type-99 records wrap CBEFF
  containers; MINEX minutiae mirror ANSI/NIST Type-9 semantics; EBTS reuses
  ANSI/NIST record-type numbering (Type-1 header + typed records).
- DB coupling by convention: `fhe-match/context.rs` mirrors
  `eel_fhe_contexts` rows, `smpc-match/session.rs` stubs an
  `eel_smpc_sessions` write via tracing, and `model-canary/fairness_rollback.rs`
  consumes `eel_fairness_evaluations` rows from `abis-perf-monitor`.

---

# Group 1 — Security & privacy

## abis-cancelable

**Purpose** (`src/lib.rs:1`): ISO/IEC 24745 cancelable biometrics — BioHashing,
random projection (Johnson-Lindenstrauss), fuzzy vault, fuzzy commitment,
revocation, renewal.

**Mechanism**

- BioHashing: `src/biohash.rs:64 BioHasher::hash` derives output bytes as `SHA-256(user_key || chunk_idx || feature pairs)`. This is keyed hashing, not textbook projection-based BioHashing — Hamming distance between hashes does not track feature similarity.
- Random projection: `src/random_projection.rs:36 RandomProjector::new` builds a key-derived matrix in [-1,+1] via SHA-256 counter mode; `project` (`random_projection.rs:80`) scales by `1/sqrt(out_dim)`. Genuine JL-style projection, PRF randomness instead of Gaussian.
- Fuzzy vault: `src/fuzzy_vault.rs:40 lock` encodes a secret-derived polynomial over template points plus chaff. But chaff x-values are sequential (`template.len()+i+1`) and trivially separable, and `unlock` (`fuzzy_vault.rs:92`) requires the secret and counts exact matches — no polynomial reconstruction from overlapping probes. Keyed point-set check, not a real vault.
- Fuzzy commitment: `src/fuzzy_commit.rs:41 commit` XORs template with a codeword, but the ECC is an admitted stub (`ecc_encode` at `fuzzy_commit.rs:114` repeats `SHA-256(secret)`); `decode` (`fuzzy_commit.rs:74`) estimates distance from a hash-difference heuristic. No real error correction.
- Revocation/renewal: `src/revocation.rs:48 revoke` appends to an in-memory ledger (CloudEvent publication documented but not implemented); `renew` at `src/renewal.rs:38` derives the new key as `SHA-256(old_key || subject || revocation_ref)`.

**Public API**: `BioHash(er)` + Hamming distance (`biohash.rs:30`);
`RandomProjector` (`random_projection.rs:36,80`); `FuzzyVault`
(`fuzzy_vault.rs:40,92`); `FuzzyCommit` (stub ECC, `fuzzy_commit.rs:41`);
`RevocationLedger`/`RevocationRecord` (`revocation.rs:48`);
`renew`/`RenewalRecord` (`renewal.rs:38`); aggregate `CancelableError`.

**Verification**: 30 tests in `tests/cancelable_test.rs` (incl. proptest):
round-trips, irreversibility smoke checks, distance-preservation bounds,
vault behavior, revocation bookkeeping, key rotation independence.

## abis-shielding

**Purpose** (`src/lib.rs:1`): irreversible Bloom-filter shielded templates;
no plaintext leaves the shielded form. Small crate that does exactly what it
claims.

**Mechanism**

- `src/shield.rs:87 Shielder::shield`: Bloom filter over template bytes with
  `hash_count` independent SHA-256 positions.
- Similarity = bit Jaccard: `ShieldedTemplate::similarity`
  (`shield.rs:33`), plus `popcount` (`shield.rs:56`).
- Parameter sizing uses standard optimal-Bloom formulas:
  `ShieldingParams::select` (`params.rs:38`), `estimated_fmr`
  (`params.rs:70`). Applying textbook Bloom FMR math to correlated biometric
  features remains an approximation.

**Public API**: `ShieldedTemplate` (similarity/popcount), `Shielder` +
`ShieldError`, `ShieldingParams` + `ShieldingParamsError`.

**Verification**: 16 tests in `tests/shielding_test.rs`: round-trips,
identical/dissimilar template separation, parameter formulas, input errors.

## abis-shielded-adjudication

**Purpose** (`src/lib.rs:1`): probe/gallery templates never appear in
plaintext for Tier-2/Tier-3 cases; Tier-1 election-critical retains plaintext
dual-review. Invariants: no raw bytes in reviewer payloads or audit logs,
TierGate as sole policy point, HSM-signed chained actions, shielded scores
within eps=0.02 of plaintext.

**Mechanism**

- TierGate: pure function `gate` at `src/tier_gate.rs:34`;
  `allows_plaintext` (`tier_gate.rs:42`).
- `ShieldedCase::new` (`src/case_shield.rs:48`): shields probe/gallery into
  Bloom filters immediately; serialization deliberately omitted (documented
  at `case_shield.rs:23`). UI hints expose only popcounts/intersections
  (`case_shield.rs:93`).
- Audit: `create_shielded_event` (`src/audit.rs:35`) delegates Ed25519
  signing/chaining to `eel-abis-adjudication`'s `HsmSigner`, adding only
  shield metrics; chain verify (`audit.rs:66`); leak assertion greps
  canonical JSON for probe hex (`audit.rs:75`).
- UI hooks: `build_payload`, `assert_no_template_leak`,
  `may_render_raw_image` (`src/reviewer_ui_hooks.rs:32,55,72`).

Crypto lives in `eel-crypto-rs`/adjudication; this crate is orchestration
plus leak assertions. The eps=0.02 claim is tested only against its own
Bloom pipeline, not a production matcher.

**Public API**: `CaseSeverity`/`ReviewMode`/`gate`/`allows_plaintext`;
`ShieldedCase` + `CaseShieldError`; `ShieldedAuditEvent` +
`create_shielded_event`/`verify_shielded_chain`/
`assert_no_plaintext_in_event`; `ShieldedReviewPayload` +
reviewer hook functions.

**Verification**: 15 tests in `tests/shielded_test.rs` + 18 inline
(`#[cfg(test)]` across tier_gate, case_shield, reviewer_ui_hooks, audit) =
33. Includes negative leak tests scanning payloads/audit JSON/UI for raw
probe hex via regex.

## abis-smpc-match

**Purpose** (`src/lib.rs:1`): SMPC matching claiming SPDZ/BGW/GMW/Yao/AB+
with Shamir sharing; lib doc states tests use a 3-party local simulation.

**Real vs simulated**: the Shamir math is real — Mersenne prime field 2^61-1 (`FIELD_PRIME`, `src/share.rs:8`), correct Lagrange interpolation at zero (`src/share.rs:80 reconstruct`), Fermat inversion (`share.rs:150`), modular exponentiation. Everything above it is simulation:

- Single process, no networking, no adversary, no MAC checking.
- `SecureDotProduct::compute` (`src/match_dot.rs:46`) splits values additively but skips Beaver triples (self-documented at `match_dot.rs:63`); each party multiplies shares locally and results are summed — not secure multiplication.
- Coefficients come from a public-seed LCG (`lcg_next`, `share.rs:169`), so "random" polynomials are reproducible.
- `verify_share` (`share.rs:120`) verifies nothing about share values.
- `Protocol`/`SmpcSession`/`SmpcPolicy` are configuration scaffolding: party-count minimums (`protocol.rs:29,40`), lifecycle with tracing-event DB stub (`session.rs:47`), threshold/adversary checks (`policy.rs:45`). No named protocol executes.

**Public API**: `ShamirShare`/`ShamirSharing`/`ShareError` (real Shamir);
`SecureDotProduct`/`DotProductResult` (simulation); `Protocol` +
`min_parties`/`validate_party_count`; `SmpcSession`/`SessionStatus`;
`SmpcPolicy`/`SmpcPolicyOutcome`.

**Verification**: 23 tests in `tests/smpc_test.rs`: split/reconstruct
round-trips, thresholds, dot-product equality with plaintext, session/policy
validation. Prove arithmetic and plumbing; cannot prove protocol security.

## abis-fhe-match

**Purpose** (`src/lib.rs:1`): homomorphic matching. The lib doc is candid:
"Interface-only... a pure-Rust toy implementation... suitable for unit
tests." In practice ciphertexts hold plaintext integers.

**Real vs toy**

- `CkksCiphertext` (`src/match_ckks.rs:21`) stores `Vec<i64>` + scale; "encrypt" is quantization. `CkksMatcher::match_1v1` (`match_ckks.rs:71`) computes cosine similarity in the clear. No ring, noise, or keys.
- `BfvMatcher::match_1v1` (`src/match_bfv.rs:50`) is clear-text XOR popcount.
- BGV/TFHE exist only as enum variants (`context.rs:26`); no code.
- `BatchMatcher::search_1n` (`src/batch_1n.rs:32`) loops the CKKS matcher; claimed SIMD packing is not implemented.
- `FheContext` (`context.rs:55`) validates realistic parameters (degree >= 1024 power-of-two, 128-bit presets, depth limits) and round-trips JSON shaped like `eel_fhe_contexts` rows — production-shaped metadata around a toy core. Value: the `CkksMatchTrait` seam for a future real library.

**Public API**: `FheContext`/`FheScheme`/`SerializedContext` (+ presets at
`context.rs:83,88`); CKKS types + `CkksMatchTrait`; BFV types;
`BatchMatcher`/`BatchMatchResult`; `FhePolicy`/`FhePolicyOutcome`.

**Verification**: 22 tests in `tests/fhe_test.rs`: encrypt/decrypt symmetry,
score parity with plaintext computation, dimension/threshold errors, batch
best-match, context validation. All pass because operations are plaintext.

## abis-pad-advanced

**Purpose** (`src/lib.rs:1`): iBeta Level 2 PAD across fingerprint
(multispectral, LivDet-family), face (challenge-response, passive texture,
deepfake), iris (pupil dynamics), voice (replay/synthesis/conversion),
enforced by `PadPolicy`.

**Signal processing or heuristics? Mostly heuristics:**

- LivDet fp PAD: pluggable `Detector` trait (`livdet.rs:39`); baseline fuses gray-transition-rate "ridge frequency" (`livdet.rs:124`) with dark-pixel "valley fill" bucketing (`livdet.rs:140`). Crude statistics, not FFT ridge analysis.
- Passive face (`passive_face.rs:49 analyse`): luma variance mislabeled LBP (`passive_face.rs:91`, self-described Stub), r>g>b skin-tone heuristic (`passive_face.rs:109`), horizontal high-frequency "moire" energy (`passive_face.rs:130`).
- Challenge-response: strongest component. Nonce replay defense (`challenge_response.rs:65 issue`), forward-chained SHA-256-PRF step sequences (`generate_steps`, `challenge_response.rs:123`), ordered verification (`challenge_response.rs:89`). Verifies declared responses only; facial-movement recognition assumed upstream.
- Deepfake (`deepfake.rs`): pure vendor boundary; `StubAdapter` (`deepfake.rs:103`) scores embedding norm / zero energy. MesoNet/RawNet2 named as future integrations.
- Multispectral (`multispectral.rs:60 analyse`): per-band mean pixel value times spoof-resistance weight; self-labeled Stub.
- Iris (`iris_pad.rs:55 analyse`): pupil-variance + glint-ratio thresholds on pre-extracted measurements.
- Voice (`voice_pad.rs:56 analyse`): sideband-energy replay score (`voice_pad.rs:100`), constant-F0-jitter TTS score (`voice_pad.rs:119`), formant-discontinuity conversion score (`voice_pad.rs:137`) on pre-extracted features.
- Policy: fail-closed APCER/BPCER budgets per required PAI species with Level-1/Level-2 coverage lists (`policy.rs:85 enforce`, species lists at `policy.rs:127`). Solid bookkeeping over non-certification-grade detectors.

**Public API**: `PadError`/`Decision`; `ChallengeVerifier` +
sequences/challenges; `DeepfakeDetector` + `VendorAdapter`/`StubAdapter`;
`LivDetVerifier` + `Detector`/`BaselineDetector`; multispectral
capture/`analyse`; `PassiveFaceVerifier`; `IrisPadVerifier`;
`VoicePadVerifier`/`VoiceAttack`; `PadPolicy`/`PolicyOutcome`/
`IbetaLevel`/`PaiSpecies`.

**Verification**: 29 tests in `tests/pad_test.rs`: nonce rejection, per-
modality decisions, attack attribution, policy violations, input validation.
Prove control flow, not detection efficacy against real spoofs.

## abis-attest

**Purpose**: no crate-level doc (only Group 1 crate without one); modules
provide unified Intel TDX / AMD SEV-SNP / dev software attestation with
nonce freshness and measurement allow-lists.

**Real vs simplified**

- Parsing is real byte-level work against published layouts: SEV-SNP 1184-byte report with correct offsets (`sev.rs:48 parse`; layout table at `sev.rs:3`); TDX 584-byte TDReport extracting MRTD/RTMR0-3/report_data (`tdx.rs:30 parse`).
- Signature verification is NOT done: `TrustedVcekStore`/`TrustedPckStore` traits (`sev.rs:67`, `tdx.rs:55`) ship only trust-all dev impls, and whole quote bytes stand in for the cert chain (`tdx.rs:90`). No ECDSA/P-384, AMD root chain, or DCAP.
- Nonces are real within scope: UUID-v4-pair 32-byte nonces, 60s window, single-use consume with replay/expiry errors (`nonce.rs:70 consume`), stale eviction.
- Policy is real logic: profile requirements, measurement allow-lists, minimum CPU SVN, software-attestation rejection outside dev (`AttestationPolicy::enforce`, `policy.rs:80`).
- Two layers: simple env-selected `verify_attestation` (`attestation.rs:31`, magic-byte checks) and fuller `AttestationVerifier` (`verify.rs:54`: nonce -> detect -> parse -> policy).

**Public API**: attestation records + `verify_attestation`/`attest_batch`
(`attestation.rs:31,127`); SEV-SNP parse/trust types (`sev.rs`); TDX
parse/trust types (`tdx.rs`); `Nonce`/`NonceStore`; `AttestationPolicy`/
`AttestationVerdict`/`ReportPayload`; `AttestationVerifier`.

**Verification**: 23 tests in `tests/attest_test.rs`: report round-trips
with synthetic fixed-layout reports, magic mismatches, nonce expiry/replay,
allow-list/SVN violations, dev-vs-prod software attestation. Chain trust
untested because stubbed.

## abis-cert-fido

**Purpose** (`lib.rs:1`): FIDO Biometric Component Certification evidence —
PAD species coverage, PAI matrix, FMR/FNMR report, tester signature *stub*
(its own words).

**Mechanism**: PAI matrix keyed by (attack_type, species, medium) with
duplicate rejection (`matrix.rs:73 add`), blocked filtering (`matrix.rs:91`),
max-APCER (`matrix.rs:98`). `EvidencePack::assemble` (`evidence.rs:54`)
enforces targets, then attaches `sign_stub` (`evidence.rs:92`) =
hex(SHA-256(component_id || fmr_le || fnmr_le)) — trivially forgeable and
does not bind the matrix or timestamps.

**Public API**: `PaiMatrix`/`PaiMatrixEntry`/`ExpectedOutcome` + enums;
`EvidencePack`/`PadReport`/`FmrFnmrReport`/`EvidenceError`.

**Verification**: 11 tests in `tests/fido_test.rs`: duplicates, target
breaches, blocked iteration, max-APCER, serialization round-trip.

## abis-cert-frvt

**Purpose** (`lib.rs:1`): FRVT participation infrastructure packaging
template extraction, verification pipeline, and report cards for 1:1, 1:N,
Quality, and PAD tracks.

**Honesty check**: packaging, not participation. 1:1 similarity is admitted
stub `toy_sim` (`frvt_1v1.rs:102`, byte-equality fraction); TAR@FAR via
impostor-score sorting (`tar_at_far`, `frvt_1v1.rs:110`) is coherent given
the toy scorer, meaningless for a real algorithm. 1:N returns fixed rates
proportional to log gallery size (`frvt_1vn.rs:25`). ACER = mean of
APCER/BPCER (`frvt_pad.rs:25`). Quality-FMR correlation equals mean/100
(`frvt_quality.rs:37`, Stub). No NIST submission format emitted; outputs are
serde structs.

**Public API**: `Frvt1v1Submission`/`Frvt1v1Report` (TAR@FAR{1e-3..1e-6});
`Frvt1vNSubmission`/`Frvt1vNReport`; `FrvtPadSubmission`/`FrvtPadReport`;
`FrvtQualitySubmission`/`FrvtQualityReport`.

**Verification**: 14 tests in `tests/frvt_test.rs`: target-FMR failure,
empty-input errors, range validation, deterministic fields.

## abis-model-canary

**Purpose** (`lib.rs:1`): blue-green canary rollout with auto-rollback;
traffic 0->10->25->50->100% with 15-minute holds, deterministic per-subject
routing via FNV-1a.

**Mechanism**: state machine over `CanaryState` with sqlx mapping
(`domain.rs:22`; steps/holds at `domain.rs:7-8`); `start_canary`
(`canary.rs:10`) with SLA limits; hold-gated `try_advance`
(`canary.rs:47`); transitions in `promote.rs:9`/`rollback.rs:10`. Traffic
split by FNV-1a(subject_uuid) % 100 vs traffic_pct (`dispatcher.rs:8
route_to_canary`; sticky and even). SLA breach scan returning typed
`RollbackReason` (`monitor.rs:8 check_sla`). Fairness guard (HARD-23)
compares slice-level canary/baseline FMR/FNMR from `eel_fairness_evaluations`
(default 0.5pp/1.0pp tolerances) — `detect_fairness_breach`
(`fairness_rollback.rs:54`), idempotent wrapper (`fairness_rollback.rs:98`).

Most production-plausible Group 1 crate: pure deterministic logic, DB/event
wiring left to callers.

**Public API**: domain types (`ModelCard`, `CanaryRecord`, `CanaryMetrics`,
`CanaryState`, `RollbackReason`); `start_canary`/`next_step`/
`try_advance`; `route_to_canary`/`select_model`; `check_sla`; `rollback`/
`promote`; fairness config + detection functions.

**Verification**: 29 tests across `tests/canary_tests.rs` and
`tests/fairness_rollback_test.rs`: step gating, routing determinism/
distribution, every breach reason, tolerance edges, idempotent rollback.

---

# Group 2 — Standards interchange

## abis-standards-ansi-nist

**Purpose** (`lib.rs:1`): ANSI/NIST-ITL 1-2011 (NIST SP 500-290 e3)
transaction codec — typed record types 1-22, 98, 99, binary parser/serialiser,
Type-99 CBEFF wrapping. Doctested.

**Mechanism**: FS (0x1C)/US (0x1F) separators per e3 section 7.3
(`transaction.rs:7-8`); parser splits records/fields, parses `RRR.NNN:` tags
(`transaction.rs:133 parse_tag`), recovers IDC, drops length field 1
(recomputed on serialize). Self-inclusive lengths resolved iteratively
(`transaction.rs:189 resolve_length`). Binary field values are hex-encoded to
avoid separator collision — a deviation: real Type-3/4 images are raw binary,
so the codec is self-compatible rather than bit-exact with stock feeds. Typed
accessors for Types 1, 2, 4, 9, 10, 13, 14, 15, 17, 99 (`types/mod.rs`),
e.g. Type-1 `TransactionInfo` (`types/type1.rs:27`).

Coverage: transaction/tag layer fully parsed; record-type semantics partial
(10 of 25 types typed); binary image handling nonstandard.

**Public API**: `NistTransaction` (parse `transaction.rs:34` / serialize);
`NistRecord`/`NistField`/`RecordType`; ten typed record models; `NistError`.

**Verification**: 28 tests in `tests/ansi_nist_tests.rs`: round-trips, tag
edge cases, malformed streams, length self-consistency, typed accessors.

## abis-standards-cbeff

**Purpose** (`lib.rs:1`): ISO/IEC 19785-1 CBEFF v3 container codec with
nested containers and format-owner registry. Doctested.

**Mechanism**: project-local wire format anchored by magic `CB EF 00 01`
(`container.rs:8`) — SBH, length-prefixed BDB, optional SecurityBlock. Not
the standard's TLV encoding nor a patron format; SBH fields mirror clause 7
and quality records clause 7.8 (`sbh.rs:3,33`), with careful big-endian
bounds-checked parsing (`sbh.rs:55 parse`). Nesting via BDB magic inspection
(`container.rs:132,138`). Registry ships four owners only
(`registry.rs:9`).

Coverage: full round-trip fidelity for its own dialect; partial conformance
to 19785-1 (semantics modeled, official encoding not followed, no patron
identifier header).

**Public API**: `CbeffContainer` (parse `container.rs:42` / serialize /
nested unwrap), `CBEFF_MAGIC`; `StandardBiometricHeader`/`QualityRecord`;
`BiometricDataBlock`/`SecurityBlock`; `FormatRegistry`; `CbeffError`.

**Verification**: 18 tests in `tests/cbeff_tests.rs`: round-trips with/without
SB, nested unwrap, truncation errors, quality arrays, registry lookups.

## abis-standards-minex

**Purpose**: no crate doc; from modules — ISO/IEC 19794-2 / NIST MINEX III
minutia template codec, conformance validation, cross-vendor interop scoring.

**Mechanism**: FMR wire format — 17-byte header (magic `FMR\0`, version,
length, finger position/view/impression, count) + 6-byte minutiae packing
type into top 2 bits of the 14-bit x coordinate (`template.rs:81 serialize`,
`template.rs:112 parse`). The minutia layout matches real ISO 19794-2 normal
format; header framing is simplified (no INCITS 378-style outer header;
project-chosen magic/version). Conformance enforces MAX_MINUTIAE=128,
500x500@500ppi, quality <= 100, position 1-10 (`validator.rs:14 validate`)
mirroring MINEX III criteria. Interop scoring implements the MINEX III
matching criterion literally — 15px proximity + 22.5-degree tolerance,
greedy one-to-one pairing (`interop.rs:26 compute_interop_score`).

Coverage: most standards-exact payload codec in the group; framing partial.

**Public API**: `MinexTemplate`/`MinexMinutia`; geometry/count constants;
`MinexValidator`/`ConformanceResult`; `VendorTemplate`/`InteropScore`/
`compute_interop_score`; `MinexError`.

**Verification**: 10 tests in `tests/minex_tests.rs`: wire round-trip,
bit-packed coordinate recovery, count/bounds/quality violations, interop
sanity.

## abis-standards-ebts

**Purpose** (`lib.rs:1`): FBI EBTS 10.0 codec — all 13 TOT codes typed,
binary serialiser/parser for transactions and responses, IDC-linked records.
Doctested.

**Mechanism**: complete `TypeOfTransaction` enum with Display/FromStr
(`tot.rs:10-80`). Wire format is project-local: compact Type-1 header
(`[rt][idc][u16 len]["TOT:..\x1fDAI:.."]`) then `[rt][idc][u32 len][hex]`
records (`transaction.rs:73 serialize`, `:108 parse`). As with ANSI/NIST,
record payloads are hex-wrapped, so this models EBTS semantics in a private
envelope, not FBI-compliant streams. Responses embed Accept/Reject/Error in
header content (`response.rs:28,38 EbtsResponse`).

Coverage: TOT taxonomy complete; interoperability with real state systems
partial.

**Public API**: `TypeOfTransaction`; `EbtsTransaction`/`EbtsRecord`;
`EbtsResponse`/`ResponseStatus`; `EbtsError`.

**Verification**: 16 tests in `tests/ebts_tests.rs`: round-trips per TOT,
response parsing, truncation errors, IDC linkage.

## abis-standards-icao

**Purpose**: no crate doc; modules cover TD3 MRZ (DG1), face/finger/iris DGs
(2/3/4), security options (DG14), active-auth key (DG15), SOD, BAC key
derivation.

**Mechanism**

- MRZ: strict 88-byte TD3 parse with `<<` name splitting and filler trim (`dg1.rs:11 MachineReadableZone`); genuine ICAO 7-3-1 check digit (`dg1.rs:78 check_digit`).
- BAC: `derive_key` (`bac.rs:29`) follows the 9303 Part 11 construction but substitutes SHA-256 for mandated SHA-1 (documented tradeoff) — keys will NOT match conformant chips until swapped. `pace_derive_key` (`bac.rs:53`) is an explicit zero-filled stub.
- SOD: minimal custom encoding instead of DER/CMS DocumentSecurityObject; passive auth compares SHA-256 digests of DG bytes against stored hashes (`sod.rs:66 verify_passive_auth`) — digest comparison real, but no X.509/CMS signature chain, so passive-auth shape without trust.
- DG14/DG15: minimal custom framings for protocol OID strings and DER blobs (`dg14.rs:13`, `dg15.rs:17`). No APDU layer anywhere.

Coverage: DG1 essentially complete; DG2-4/14/15 structurally present with
simplified encodings; SOD/BAC partial; PACE stub.

**Public API**: `MachineReadableZone` + `check_digit`;
`DocumentSecurityObject` (parse `sod.rs:20`, encode, passive-auth);
`derive_key`/`KeyMode`/`pace_derive_key`; `SecurityOptions`;
`ActiveAuthPublicKey`; `IcaoError`.

**Verification**: 21 tests in `tests/icao_tests.rs`: MRZ extraction and
filler handling, known-answer check digits, SOD tamper detection, BAC
parity adjustment, DG parse errors.

## abis-standards-interpol

**Purpose**: no crate doc; modules provide an NCB-to-NCB search-request
envelope carrying ANSI/NIST-linked records and crime/person entities.

**Mechanism**: `NcbCode` validation (3 ASCII letters, uppercased,
`envelope.rs:9`); envelope with sender/receiver codes, purpose (criminal/
person/lost-doc/stolen-vehicle), response windows (0/24/72h)
(`InterpolEnvelope`, `envelope.rs:67`), opaque records carrying
`nist_type1_idc` links. Serialization is length-prefixed JSON
(`envelope.rs:108`) — no official INTERPOL/I-24/7 wire format; message IDs
are nanosecond-time strings (`uuid_string`, `envelope.rs:136`). Entity
linkage is struct composition (`entity.rs:28 link_crime_to_person`).

Coverage: least standards-grounded of the six; treat as internal messaging
schema themed on INTERPOL workflows.

**Public API**: `NcbCode`; `MessagePurpose`/`ResponseWindow`;
`InterpolEnvelope`/`EnvelopeRecord`; entity types + linkage; `InterpolError`.

**Verification**: 9 tests in `tests/interpol_tests.rs`: code validation,
round-trips, truncation errors, linkage.

---

# Closing notes

## Stubs and simplifications

1. **SMPC**: real Shamir arithmetic; everything above it simulates. No Beaver
   triples (self-documented), LCG coefficients, ineffective `verify_share`;
   protocol names are labels only.
2. **FHE**: no homomorphic encryption exists; ciphertexts are plaintext
   containers. Value is the trait seam and context metadata for a future
   library swap; BGV/TFHE have no code.
3. **PAD**: decision plumbing, replay defense, and policy bookkeeping sound;
   detectors are statistic/threshold heuristics, several self-labeled Stub.
   Not comparable to certified commercial PAD.
4. **Attestation**: layout parsing and policy real; quote signature chains
   trusted unconditionally via dev stores.
5. **Cancelable**: BioHash is keyed hashing (not projection BioHashing);
   vault unlock consumes the secret instead of reconstructing it; fuzzy-
   commitment ECC is a stub with no true correction.
6. **Cert harnesses** (fido/frvt): evidence assembly and report cards only;
   tester signatures and similarity metrics are stubs. Artifacts, not
   certifications.
7. **Model canary** is the most deployable Group 1 crate: deterministic
   logic with typed rollback reasons, pending DB/event wiring.

## Standards coverage summary

| Standard | Crate | Coverage | Notes |
|---|---|---|---|
| ANSI/NIST-ITL 1-2011 | ansi-nist | Partial | Transaction layer complete; 10/25 record types typed; binary payloads hex-wrapped |
| ISO/IEC 19785-1 CBEFF | cbeff | Partial | SBH semantics aligned; wire format project-local, custom magic |
| ISO 19794-2 / MINEX III | minex | Mostly | Minutia layout matches ISO 19794-2; simplified header; real conformance checks and match criterion |
| FBI EBTS 10.0 | ebts | Partial | All 13 TOTs modeled; private binary envelope, hex-wrapped records |
| ICAO 9303 | icao | Mixed | DG1/MRZ + check digits complete; BAC SHA-256-for-SHA-1 substitution; PACE stub; SOD lacks CMS chain |
| INTERPOL interchange | interpol | Minimal | Internal JSON envelope; no official format |

General pattern across both groups: strong typed APIs, thorough round-trip
testing, honest inline labeling of stubs — but cryptographic and detection
cores are largely pedagogical simulations awaiting library integrations, and
the standards codecs prioritize self-consistent formats over bit-exact
interoperability.
