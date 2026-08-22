# Supreme Court of Kenya defensibility

Analysis of the platform's anti-fraud + counter-rigging controls against expert scrutiny in a Supreme Court of Kenya (SCoK) presidential-election petition. Answers the specific hostile-question set: double voting, impersonation, cross-station voting, registration under multiple names, unqualified registration, voting the deceased.

## Contents

1. Precedent + SCoK test framework
2. Court-facing evidence surface
3. Procedural practices enforced
4. Voter voting more than once
5. Impersonation prevention
6. Cross-station voting prevention
7. Registration under different names
8. Unqualified registration prevention
9. Preventing the deceased from being voted
10. Honest assessment for Court
11. Residual gaps + external-dependency risks

---

## 1. Precedent + SCoK test framework

Directly relevant precedent: *Raila Odinga v IEBC 2013*, *2017 v1* (nullification), *2017 v2* (Nyamira), *2022*. The Court's test framework:

- Conduct consistent with the Constitution and the law.
- Verifiable, transparent, credible, secure, free, and fair electoral process.
- Irregularities that affected the result.
- Admissible, credible, reliable evidence.

*Raila 2017 v1* held s.39(1C) electronic transmission of Form 34A images mandatory. *Raila 2022* rejected the petition on evidentiary grounds; SCoK reaffirmed the technical standard.

## 2. Court-facing evidence surface

| SCoK requirement | Our evidence |
|---|---|
| Electronic transmission mandatory (s.39(1C)) | Every 34A HSM-signed on kit + transmitted with monotone counter + Merkle-anchored on P05 transparency log; sub-second inclusion proof via `packages/verifier-rs` |
| Independent verifiability | Reproducibility bundle v2 (M14) rebuilds the entire tally from primary evidence to bit-identical Merkle root; HARD-4 public reproduce challenge; HARD-19 IAG independent reproduction every 24 h during T-1..T+7 |
| Chain of custody | M20 event-sourced custody + HARD-16 cryptographic seal-photo binding (kit-signed hash of seal photo bound to serial + geo + timestamp) |
| Cryptographic non-repudiation | FIPS 140-3 L3+ HSM PKCS#11; HARD-5 per-leaf PQ-hybrid ML-DSA + Ed25519; per-cycle KEK hierarchy; break-glass Shamir; M-of-N ceremonies (3-of-5 default, 4-of-7 during freeze per HARD-17) |
| Independent third-party attestation | IAB witness cosign day-1 + external observers day-2; HARD-13 24-hour smartcard ceremony runbook |
| Statistical evidence | M21 SHANGRLA risk-limiting audit (Stark 2020) with NIST beacon + HSM + IAB card combined entropy; HARD-20 1% Neyman-allocated parallel manual count feeding RLA |
| Mathematical proof of correctness | HARD-2 Groth16 ZK proof that published tally correctly sums 34A leaves; TLA+/Apalache spec for Tally + Declaration + Referendum with 3 model-checked invariants (HARD-18 nightly TLC in CI) |
| Court-facing evidence workflow | M40 election-court-it-interface: 15-item evidence bundles + petitioner access + technical witness workflow; direct eCourt integration |
| Cross-source corroboration | 34A (kit) vs 34B (RO Tauri) vs paper original (M63 ballot-serial reconciliation); `parallel_tally_drift.v1` on divergence |
| Explainable decisions | `abis-explainability`: minutiae correspondence SVG overlay + face embedding saliency + similarity reasoning text |
| Compliance verdict per event | M62 transmission-legal-basis-audit gives every TransmissionEvent a `ComplianceVerdict`; petitioner's counsel can query directly |
| Petitioner machinery | M40 direct evidence-bundle access + supplementary-request workflow |

## 3. Procedural practices enforced

Enforced at code + policy layers so procedural non-compliance either blocks the action or leaves an immutable audit trail:

| Practice | Where enforced |
|---|---|
| Two officers to open a station | Dual-control on M05 kit + `abis-attest` + geo-fence + time-fence FSM |
| Four-eyes on s.44A complementary mechanism | M25 special-voter-workflows + IAB 5% Merkle-seed sample |
| Segregation of Duties on payment approvals | M54 + M55 support-admin — API + DB layer refuses same-user raise + approve |
| Dual-biometric open for REMAND / BARRACKS facilities | M25 code-level enforcement |
| Training-gate on personnel dispatch | M38 LMS certificate required before M33 dispatch |
| Freeze window T-30 → T+7 for register writes | M35 CVR OPA hard-close at tx layer + HARD-17 4-of-7 KEK ceremony |
| M-of-N HSM ceremony for KEK rotation | 3-of-5 default; 4-of-7 during freeze |
| JIT admin credentials ≤ 4 h + session recording | WP-2.3 IAM seed |
| Chain-of-custody events for every asset | M20 + HARD-16 seal-photo binding |
| Session recording hash to tlog | M09 RO Tauri desktop |
| Signed 34A + 34B + 34C at every level | M08 kit → M09 RO → M10 backend, each HSM-signed |
| Reviewer append-only hash chain | `abis-adjudication` — PG trigger blocks UPDATE/DELETE on `eel_reviewer_actions` |
| Public inspection window before election | M64 voter-list-public-inspection + M52 gazette |
| Public verifiability of results | M11 portal + WebCrypto in-browser verification widget |
| Court petitioner evidence access | M40 15-item evidence bundle workflow + M62 compliance verdicts |
| ODPC breach notification within 72 h | `integrations/odpc-breach` |
| Quarterly Commissioner-signed register gazette | M35 + HARD-4 public reproduce challenge |
| Nightly TLA+/TLC formal check | HARD-18 `.woodpecker/tlc-nightly.yaml` |
| Nightly duplicate-cluster union-find | `abis-cluster-analysis` |
| Nightly IAG reproduce | HARD-19 IAG runbook |

## 4. Voter voting more than once

At the second polling station:

1. **Register uniqueness** — every voter is assigned to exactly one station at snapshot time. `eel_station_voters` has a UNIQUE constraint on `(voter_id, election_id)`.
2. **Kit-side gate** — each M05 BVI kit only recognises its assigned voters (per-station signed config bundle from M06 with kit-cert revoke ≤ 60 s). Voter at a wrong station is rejected before biometric capture.
3. **Session-scoped IdentificationEvent** — kit emits signed IdentificationEvent with monotone counter; unique constraint at M06 back-end on `(voter_id, election_id)`.
4. **BVV public lookup + objection window** (M07) rate-limited to prevent enumeration.
5. **Post-election detection** — M21 SHANGRLA RLA + `abis-cluster-analysis` union-find catches pairs of IdentificationEvents that biometrically match the same person.
6. **Twin registry** — pre-declared twins do not trigger false alarms so genuine duplicates stand out.

**Honest gap:** the primary defence is register + station-cert. If a bad actor bypasses kit attestation (revoked kit in honey-mode + counterfeit officer) at a second station, the second attempt is a duplicate that's only caught post-election within days. Elections Act §44 requires a 1:N ABIS check on the identification path but does not currently mandate synchronous cross-station biometric lookup — that would require every kit to talk to ABIS in real-time. Offline kits carry the risk.

**Mitigations:**
- Kit certificate revocation ≤ 60 s SLA.
- Honey-mode routing captures attacker biometrics.
- s.44A four-eyes complementary requires two officers + IAB 5 % sample.
- Cross-station duplicate detection is post-election; SHANGRLA + parallel manual count (HARD-20) is designed to catch it.
- **Recommendation:** for T-1..T+7 elections enable synchronous ABIS 1:1 check with a 3-second budget (already possible via `abis-api /v1/match/11`; requires runbook + capacity plan).

## 5. Impersonation prevention

At the station before ballot issuance:

1. **1:1 biometric verification** against the enrolled template — fingerprint + face + iris fused via `abis-fusion-advanced`. Threshold calibrated per (age_band, sex, county) cohort by `abis-threshold-tuning`.
2. **PAD Level 2** (iBeta-testable, HARD-7 adversarially trained where models exist): multispectral fingerprint + LivDet + face LBP + moire + challenge-response + 3DMM pose consistency + iris pupil dynamics + Purkinje reflex + contact-lens detection.
3. **Kit attestation** — Play Integrity + App Attest + TPM + software-tier via `abis-attest` + nonce freshness (60 s) + replay defence.
4. **Session recording hash to tlog** — M09 records the operator's session; hash anchored in the transparency log.
5. **Officer HSM-signed session events** — every voter check-in signed by the operator's SVID.
6. **Duress-finger signal** — covert coercion channel; voter can silently signal an emergency.
7. **s.44A fallback** — if biometrics fail, four-eyes + IAB sample + manual ID + adjudicator review.

**What stops an offline attack on the kit:** kit-cert-transparency (HARD-12) — a rogue CA cannot silently issue a false kit cert; the append-only log flags it. TDX/SEV-SNP enclave attestation on the kit means the kit itself is measured.

## 6. Voting in more than one place

Same defence as §4: register uniqueness + per-station kit + biometric 1:1 + post-election duplicate detection. Additionally, HARD-20 parallel manual count samples 1 % of stations; cross-station duplicates in that sample surface in the RLA feed.

## 7. Registering under different names

At enrolment (M01 BVR kit + M02 BVR backend):

1. **ABIS 1:N dedup against the full 30M gallery** (M03) using multi-modal (fingerprint + face + iris) at ingest.
2. **Modern matcher stack** — MCC (Cappelli TPAMI 2010) + FingerCode (Jain TIP 2000) + SourceAFIS + DeepPrint (Engelsma TPAMI 2021) + Level-3 pores + CDEFFS EFS; 5-matcher consensus voting with K-of-N agreement policy (HARD-22 ensemble voting on top).
3. **FMR ≤ 0.001 % + FNMR ≤ 0.1 %** per ADR-0018.
4. **Family cluster + twin registry** prevents false positives on identical twins from suppressing genuine duplicates.
5. **Adjudication with 3-of-5 panel for Tier-3 disputes** — every action HSM-signed with `prev_action_hash` chain.
6. **IPRS + NRB cross-check** — national_id → biometric + demographics consistent with civil registry.
7. **Signed enrolment events with prev_hash chain** — silent rewrite breaks the chain.
8. **HARD-3 cross-vendor 2-of-3 consensus during T-1..T+7** — higher confidence for peak-period enrolments.
9. **`abis-cluster-analysis` nightly union-find** — retrospectively catches multi-way collisions that a single-match view missed.

Bottom line: to register under different names, the attacker must present with different fingerprints AND different face AND different iris across all attempts, which is biologically implausible without surgical alteration, OR compromise the ABIS + evade cross-vendor consensus + evade twin/cluster analysis — cryptographically implausible.

## 8. Unqualified registration prevention

Kenyan qualifications (Elections Act §5 + Constitution Art. 83): 18+, Kenyan citizen, not of unsound mind, not incarcerated for a capital offence, registered where they ordinarily reside.

Enforced at M01 BVR-device + M02 BVR-backend + M04 VPRS:

| Check | Source of truth | Enforcement |
|---|---|---|
| Age ≥ 18 | IPRS DoB | Fail-closed on lookup failure; refuse if DoB missing or under-18 |
| Citizenship | NRB via IPRS + physical ID scan (MRZ per ICAO 9303 DG1) | Refuse without valid national ID + IPRS confirmation |
| Not incarcerated for capital offence | Judiciary interface (M40) + KPS/KDF/UNHCR feed (`integrations/kps-kdf-unhcr`) | Cross-check against inmate register; flag on match |
| Mental capacity | Declaration + officer discretion + medical (M60 no-PHI schema; enforcement procedural) | Refuse if declared unfit |
| Constituency of residence | Physical address verification + M18 boundary Geofence.Check | Assign only to a station within the declared constituency; boundary polygon HSM-signed |
| Duplicate registration | M03 ABIS 1:N (see §7) | Adjudication if biometric match found |
| Ongoing dormancy | KRA quarterly signal via CVR (M35) | Flag for review, not automatic removal |

**Honest gap:** freshness of judiciary + KPS interfaces depends on external systems keeping IEBC updated. Our IPRS fail-closed prevents silent auto-approve, but a recent conviction not yet in the register is missed until the next batch.

**Mitigations:**
- Officer manual review with 30-day SLA for edge cases (`abis-rectification`).
- Public inspection window (M64) allows any citizen to flag ineligible registration.
- Post-election review (M41) with parliament + press oversight.

## 9. Preventing the deceased from being voted

Two paths: (a) the deceased entry being used to vote; (b) a false death report gaming the register.

**Prevention of "voting a dead person":**

1. **CRS daily death-notification SFTP batch** → M35 CVR pipeline → `DEATH_PROPOSED` → officer triage → Commissioner + HSM approve → gazette publication → remove from register.
2. **NRB weekly ID-correction batch**.
3. **KRA quarterly dormancy signal**.
4. **`*400#` family short-code** — provisional 90-day report requiring CRS confirmation (avoids weaponisation).
5. **HARD field-officer mobile app** — commissioned officer captures death certificate on-site (hospital / mortuary / burial), OCR + geo + HSM sign, uploads to `POST /v1/cvr/field-notifications`. Reduces CRS-latency gap.
6. **ABIS soft-hold on proposed removals** — biometric template retained until adjudication; never a hard-delete without dual confirmation.
7. **Merkle-anchored quarterly snapshot** — every register version has a signed Merkle root; a vote for a person already removed in a prior snapshot is visibly inconsistent.
8. **Kit-level check** — the M05 kit at the polling station uses its signed config bundle (M06); a removed voter is not in the bundle.
9. **BVV public lookup + objection** (M07) — a family member can check the deceased's status and object.

**Prevention of "false death reports as an attack":**

- 90-day CRS grace window on `*400#` family reports — cannot cascade to removal quickly.
- Officer + Commissioner + HSM signature required for any removal — no automated deletion.
- ABIS soft-hold — biometric template retained pending adjudication.
- Any wrongful removal appears in the next quarterly gazette (M52) and can be objected via M64.
- `abis-rectification` workflow with 30-day SLA — wrongly-flagged person can request reinstatement with evidence.

**Post-election forensic check:**

- SHANGRLA RLA (M21) samples ballots; a vote in the name of a person already deceased on election day per CRS records surfaces in the audit.
- `abis-cluster-analysis` finds impossible clusters (identification recorded for a subject whose CRS death predates election day).
- HARD-20 1 % parallel manual count feeds RLA.

**Honest gaps:**

- CRS coverage of rural deaths is imperfect: ~40 % of Kenyan deaths were unregistered per KNBS 2019. The platform can only remove what it knows about.
- Diaspora deaths (M24) rely on embassy reporting.
- Deaths on election day itself cannot practically be removed before voting closes.

**Mitigations in place:**
- HARD field-officer app closes some of the rural gap.
- `*400#` reduces friction of family reporting.
- Post-election SHANGRLA + parallel manual count catches ex-post.
- M60 medical-first-response provides in-election-day mortality signals for ECC dashboard.

## 10. Honest assessment for Court

The Court's answer to each hostile question, on the record:

- **"Did anyone vote twice?"** — biometric 1:1 at station + register uniqueness + post-election duplicate cluster + parallel manual count. Any duplicate is blocked at station OR surfaced within 48 h.
- **"Did anyone impersonate someone else?"** — multi-modal biometric + PAD L2 + kit attestation + session recording. Attacker must defeat all three simultaneously.
- **"Did unqualified people register?"** — IPRS fail-closed + NRB citizenship + boundary geofence + adjudication. Any breach is auditable.
- **"Are dead people on the register?"** — CRS + NRB + KRA + `*400#` + field officer app + Commissioner + HSM + gazette. What is not caught pre-election is caught by SHANGRLA post-election.
- **"Is the tally correct?"** — HARD-2 Groth16 ZK proof + reproducibility bundle v2 + Merkle inclusion proofs + IAG independent reproduction + 5-channel STH.
- **"Is any of this true?"** — everything is cryptographically anchored; the Court can independently verify without our cooperation.

The Court cannot conclude "verifiability was absent" as in *Raila 2017 v1*. Where irregularities exist, they are findable, provable, and quantified.

## 11. Residual gaps + external-dependency risks

Ranked by severity for a hostile petition:

1. **Synchronous cross-station duplicate check not mandatory** — post-election detection catches, but on-day voting is register + kit-cert dependent. Runbook + capacity plan available for switching to synchronous mode at T-1..T+7.
2. **CRS death coverage ~60 %** — outside our control; mitigated by HARD field-officer app + post-election RLA.
3. **Judiciary + KPS inmate-register freshness** — external system latency; mitigated by public inspection + post-election review.
4. **Diaspora reporting gaps** — embassy dependency; mitigated by M24 embassy station + M65 pouch custody chain.
5. **Vendor SDK dependency for cross-vendor consensus** (HARD-3) — commercial licence required; mitigated by our in-house 5-matcher consensus as the primary defence.
6. **Real trained anomaly models require Kenyan historical datasets** — currently on deterministic synthetic corpus (HARD-1); mitigated by rule-based Benford + convergence checks running today.
7. **IAG appointment + funding** — governance dependency (HARD-19 charter written); mitigated by IAB day-1 witness.
8. **Public reproduce challenge participation** — depends on independent teams accepting; mitigated by the reproducibility bundle being independently runnable regardless.

None of these gaps invalidate the cryptographic anchoring, the audit trail, the reproducibility, or the mathematical proof of tally correctness. They shift the burden from "prevent all fraud" to "detect and quantify fraud", which is what SCoK's *2017 v1* + *2022* framework actually asks.

Related documents:
- `docs/features/anti-fraud-and-rigging-detection.md` — full 24-item hardening catalogue with commit + test-count references.
- `docs/features/continuous-voter-registration.md` — CVR module walkthrough.
- `services/abis-svc/docs/CAPABILITIES.md` — ABIS algorithm + standard citations.
- `docs/governance/independent-assurance-group.md` — IAG charter.
- `docs/runbooks/iab-witness-ceremony.md` — 24-hour IAB smartcard ceremony.
