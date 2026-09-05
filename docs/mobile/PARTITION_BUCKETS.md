# Partition Buckets — Deduplication at Scale (1:N Matching)

Standard for universal 5-axis audit: `where`/`when`/`who`/`why`/`how` applies
here — the operator selects `what` (which partitions to search) and `why`
(justification for partition selection) is logged via `AuditService`.

## Scale baseline
- 100,000,000 fingerprints (10 fingers per 10M persons, full NBIS DB)
- Unpartitioned 1:N search ≈ 10 hours (sequential FAP60/AntIris comparison)
- Partitioned 1:N search ≈ 10 seconds (500K partition → 200 candidates)

---

## Dimension catalog

### 1. Core dimensions (always selected, never deselectable)

| # | Dimension | Key values | Reduction | Search time impact | Notes |
|---|---|---|---|---|---|
| 1.01 | Gender | `Male` / `Female` / `Other` | 2–3x | Always on | From IPRS / registration |
| 1.02 | Finger position | 10 positions (LT, LI, LM, LR, LL, RT, RI, RM, RR, RL) | ~10x | Always on | Capture device reports finger |
| 1.03 | Age bucket | `0-17` / `18-35` / `36-55` / `56-75` / `76+` | 5x | Always on | From IPRS birth year |

These 3 dimensions alone: 3 × 10 × 5 = **150 partitions**
→ 100M / 150 = **667K candidates/partition**
→ 667K × 0.5ms (minutiae compare) ≈ **333s (5.5min)** for full 1:N

---

### 2. Geography / registration station (high recommendation)

| # | Dimension | Key values | Reduction | Selection impact |
|---|---|---|---|---|
| 2.01 | County / station | `NairobiWest` / `NairobiEast` / `Kisumu` / `Mombasa` / `Nakuru` / `Eldoret` / `Other` | 7x | Operator selects based on station polygon |

This is the single most powerful dimension (after finger). When combined with gender/age/finger:
150 × 7 = **1,050 partitions** → **95K candidates/partition** → **48s 1:N search**

**Operator selection**: `EelChips` showing county code from `LocationService.polygonValidated()`; operator can override (e.g., search all counties when diaspora suspected).

---

### 3. Dominant hand (medium recommendation, always on unless ambiguous)

| # | Dimension | Key | Reduction |
|---|---|---|---|
| 3.01 | Dominant hand | `Left` / `Right` / `Ambidextrous` | 2x (ambidextrous = both hands, 1.5x avg) |

This is a routing optimization. If the probe finger is `LEFT_INDEX` and the hand is `LEFT`, the partition only includes `LEFT` hand templates. If `AMBIDEXTROUS`, both hands included.

---

### 4. Ridge quality / image quality (medium recommendation, operator selectable)

| # | Dimension | Key | Reduction | Selection impact |
|---|---|---|---|---|
| 4.01 | Ridge quality (NFIQ) | `HIGH` (1-2) / `MEDIUM` (3) / `LOW` (4-5) | 3x | Operator selects minimum quality threshold |

When searching: only search partitions with quality ≥ selected threshold. This excludes low-quality templates that are unlikely to match cleanly, reducing false positives and search time simultaneously.

---

### 5. Diaspora status (high recommendation, always on)

| # | Dimension | Key | Reduction |
|---|---|---|---|
| 5.01 | Diaspora status | `InCountry` / `Diaspora` / `Newcomer` | 2x |

From IPRS / registration records. Diaspora fingerprints may have different patterns (climate effects on ridge clarity, occupational wear patterns differ by region).

---

### 6. Enrollment era / template version (high recommendation, always on)

| # | Dimension | Key | Reduction |
|---|---|---|---|
| 6.01 | Enrollment year | `2017-2019` / `2020-2022` / `2023-2025` / `2026+` | 4x |

Handled by versioning the template metadata (not the raw template). Old enrollments use earlier sensor generations (`FAP60` vs older models) — comparison across generations introduces noise.

---

### 7. Recent duplicate flag / bloom filter (system-level, always on)

| # | Dimension | Key | Reduction | Mechanism |
|---|---|---|---|---|
| 7.01 | Recent match cache | Per person: last match timestamp | Variable | Skip partitions where `AuditService` shows a match within 30 days |

This is a caching layer, not a partition. Before searching a partition, check `AuditService` replay: if this `actorId` recently matched a `person_id` in that partition, skip it (the duplicate is already resolved at audit time, not biometric time).

---

### 8. Disability / missing digit flags (operator selectable, always included if present)

| # | Dimension | Key | Reduction | Selection impact |
|---|---|---|---|---|
| 8.01 | Missing digit exclusion | Per finger: present / absent | Variable | If `LEFT_INDEX` is missing, search excludes `LEFT_INDEX` partitions |

This is a routing exclusion. The operator selects which fingers to search; if a finger is missing (from `PersonMetadata.missing_digits`), that partition is automatically excluded.

---

### 9. Operator identity / kit identity (system-level, audit only)

| # | Dimension | Key |
|---|---|---|
| 9.01 | Kit / device ID | Per `Fap60Driver.init()` device handle |
| 9.02 | Station polygon | `LocationService.polygonValidated` result |

These are audit dimensions, not partition dimensions. Every search result includes `who` (actor) and `where` (station polygon) for the 5-axis audit.

---

### 10. Cross-modal partitions (advanced, future)

| # | Dimension | Key | Cross-modal |
|---|---|---|---|
| 10.01 | Iris template index | `LEFT_EYE` / `RIGHT_EYE` | Search iris partitions when fingerprint fails (2-factor enrollment only) |
| 10.02 | Signature / handwriting | `CAPTURED` / `NOT_CAPTURED` | Cross-reference with signature verification |
| 10.03 | Voice biometrics | `ENROLLED` / `NOT_ENROLLED` | Not currently implemented |

---

## Compound partition key (final format)

```
{GENDER}|{AGE_BUCKET}|{GEO_PARTITION}|{FINGER_POSITION}|{DOMINANT_HAND}|{RIDGE_QUALITY}

Example:
  MALE|36-55|NairobiWest|LEFT_INDEX|RIGHT|MEDIUM
```

This is the compound key used by `PartitionedGallery` to route 1:N queries.
The key is computed from:
- `Gender` (IPRS / registration form)
- `AgeBucket` (IPRS birth year → age at enrollment)
- `GeoPartition` (`LocationService.stationPolygon` or `Person.station_id`)
- `FingerPosition` (capture device reports which finger is scanned)
- `DominantHand` (IPRS / registration form, or inferred from finger selection pattern)
- `RidgeQuality` (`FingerprintExtractor.extract()` quality score, or probe `NFIQ` result, bucketed by `NfiqQuality.to_quality()`)

---

## Operator selection UI (implemented in `BiometricOutboxPanel` / `EnrolmentPanel`)

The operator selects/deselects partition filters via `EelChips` (multi-select chips) before initiating upload or 1:N search. The selection is logged to `AuditService` (5-axis: `what` = selected partition filters; `why` = operator justification; `how` = `PartitionedGallery.search()` method + proof hash).

### Selection states

| Filter chip state | Meaning for 1:N search | Impact |
|---|---|---|
| ✅ Gender selected | Search all gender partitions |
| ❌ Gender excluded | Search only selected gender |
| ✅ Age bucket selected | Search all age buckets |
| ❌ Age bucket excluded | Search only selected bucket |
| ✅ Geo selected | Search all stations |
| ❌ Geo excluded | Search only selected station |
| ✅ Quality ≥ MEDIUM | Search HIGH + MEDIUM |
| ❌ Quality LOW excluded | Skip LOW-quality partitions |

### Default selection (operator can override)

By default, all filters are `✅ SELECTED ALL` (search full compound key). The operator deselects filters to expand or narrow the search space. For example:

- Narrow search (fast): deselect `All Geo` → select only `NairobiWest` (10x faster)
- Broad search (thorough): select `All` for gender + age + geo + quality (100x slower, but fewer false negatives)

---

## Deduplication time estimates (per 100M fingerprints)

| Filter configuration | Partitions | Candidates/part | 1:N time | Reduction factor |
|---|---|---|---|---|
| No partitioning | 1 | 100,000,000 | 10 hours | 1.0x |
| Gender + Finger only | 30 | 3,333,333 | 20 minutes | 30x |
| Gender + Finger + Age | 150 | 666,666 | 4 minutes | 150x |
| Gender + Finger + Age + Geo | 1,050 | 95,238 | 30 seconds | 1,050x |
| Full compound (all 6 dims) | 6,300 | 15,873 | 5 seconds | 6,300x |
| Full compound + recent-match cache | ~3,150 | 7,936 | 2.5 seconds | 12,600x |

The estimates assume 0.5ms per minutiae comparison (FAP60/AntIris `match()` call with NBIS `bozorth3` scoring on device CPU). With 5-second 1:N at full compound, the system supports real-time verification at kiosk scale.

---

## Implementation in code

### `docs/mobile/POLL_CLERK.md` (poll-clerk reference)
- Section "Audit Reference" includes partition key format
- `AuditReference.auditSurface()` includes `where` = `stationPolygon`, `who` = `actorId+actorRole`, `when` = `whenUtc` (5-axis audit preserved)

### `docs/mobile/BVR_KIT.md` (bvr-kit reference)
- `BiometricCapturePolicy` includes `environmentHint` (indoor/outdoor) which affects quality thresholds (indoor = 60/55; outdoor = 50/40 for fingerprint/iris)
- `DualBiometricCapture.enroll()` produces audit entry + outbox entry; the outbox uses `BiometricSubmissionOutbox._checkLocalDedup()` which compares templates via driver `matchFingerprint()` / `matchIris()` (minutiae/pattern comparison, not SHA-256 equality)

### `packages/eel-core-flutter/lib/src/audit/audit_model.dart`
- `AuditEntry` includes `auditId` (UUID7), `actorId`, `actorRole`, `action`, `target`, `reason`, `whereGps`, `whenUtc`, `howMethod`, `howProofHash`, `howSignature`, `appName`, `isOffline`, `sessionId`
- 5-axis preserved for every `AuditService.log()` call (including `enqueue()` results, `uploadAll()` results, `deduplicated` rejections)

### `services/biometric-edge-svc/apps/bvr-kit/lib/src/core/drivers/dual_biometric/dual_biometric_capture.dart`
- `DualBiometricCapture.enroll()` calls `_audit.log()` (full 5-axis) + `_outbox.enqueue()` (local dedup via `matchFingerprint()` / `matchIris()` — real NBIS algorithms)
- `BiometricSubmissionOutbox.enqueue()` runs local dedup before submission (prevents same-device duplicate enrollments)
- `uploadAll()` drains queued submissions to `/v1/bvr/enrol` (BFF endpoint, real HTTP POST with auth token + retry)

---

## References
- NBIS v5.0.0 source: `vendor/nbis/` (mindtct, bozorth3, nfiq, commonnbis — 72 .c files, real NIST algorithms, public domain per 17 USC 105)
- FAP60 SDK: `FUFingerSDK.java` (USB scanner) + `FPAPI.java` (`CompareTemplates()` returns 0–100 score; `DEF_MATCH_SCORE=45` = FAR ≤ 0.01% threshold)
- AntIris SDK: `AntUSBCamera.jar` (`IrexCaptureDialog.java`) — iris capture + `IrisIso.generate()`; comparison through texture/pattern matching
- `docs/mobile/AUDIT_AND_GEOGRAPHY.md`: universal 5-axis standard (where/when/who/why/how) + compliance (DPA 2019, Elections Act, 5-year retention)
- Design-system: `eel_design` widgets (`EelScaffold`, `EelButton`, `EelStatusChip`, `EelBanner`, etc.) applied across all 14 mobile apps
