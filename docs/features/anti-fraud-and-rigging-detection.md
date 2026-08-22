# Anti-fraud, counter-fraud, and rigging detection

Comprehensive catalogue of what's built into the platform to detect and prevent election fraud + rigging + coordinated inauthentic behaviour, organised by threat surface. Every entry cites the module, crate, or file path where the capability lives. Ends with 24 hardening recommendations, each of which is being implemented in the follow-up work packages HARD-A through HARD-E.

## Contents

1. Threat model
2. Ballot-tally rigging (T-0)
3. Voter register rigging (pre-election)
4. Kit + device fraud
5. Voter fraud, coercion, impersonation
6. Insider fraud (staff, officers, adjudicators)
7. Transmission / signal-path rigging
8. Financial fraud
9. Coordinated inauthentic behaviour (CIB) + info-ops
10. Cryptographic + supply-chain rigging
11. Post-election audit + petition
12. Chaos + resilience testing
13. 24 hardening recommendations
14. Implementation status

---

## 1. Threat model

The platform assumes a well-resourced state or non-state adversary willing to spend significant time and money, with insiders at up to 20% of any single service role, physical access to a small percentage of kits, and the ability to observe (but not silently rewrite) five independent publication channels. Defence-in-depth spans cryptographic, architectural, statistical, operational, and forensic layers so no single compromise flips the tally.

## 2. Ballot-tally rigging (T-0 election night)

### Detection (module M26 anomaly-fraud-detection, services/ai-platform-svc)

| Layer | Algorithm | Reference |
|---|---|---|
| Telemetry Isolation Forest | per-kit send rates + dwell times + retransmit patterns | Liu, Ting, Zhou ICDM 2008 |
| Cross-source comparison | 34A (kit) vs 34B (RO Tauri) vs paper original (M63) | own |
| Benford's law first-digit | national + per-station count distributions | Beber & Scacco AJPS 2012 |
| Second-digit Benford | tighter distribution test on tally digits | Rozenas Statistica Sinica 2017 |
| Cryptographic canary | sealed pre-seeded ballot serials known only to auditor set | own |
| Convergence divergence | kit stream vs paper 34B compile totals | own |
| Kit L1 on-device detector | pre-egress tamper detection | own (M28) |
| Reconciliation holds | visible-hold-not-silent-accept on serial mismatch (M63) | own |

Every layer emits `parallel_tally_drift.v1` or a modality-specific event when triggered.

### Architectural prevention

- **TLA+/Apalache formal spec** for Tally + Declaration + Referendum (`services/rts-svc/docs/tla/`). Three invariants including `ExactlyTwentyFourCounties` and `NoDeclarationWithoutBothThresholds`.
- **Exactly-once Kafka-transactional tally consumer** — offsets + PG rows atomic; single-broker rewrite impossible.
- **HSM-countersigned declarations** (M51) via per-cycle KEK from `eel_kek_generations` + Commissioner smartcard.
- **prev_hash chain** per event stream — silent rewrite breaks the chain.
- **Merkle inclusion + consistency proofs** — every 34A / 34B / 34C is a leaf with a proof; verified by `packages/verifier-rs` + WASM in the portal.

## 3. Voter register rigging (pre-election)

- **County-partitioned versioned voters** with audit-shadow triggers (M02 bvr-svc).
- **Merkle root per register snapshot** (M35 CVR) — Commissioner HSM-signed and anchored to the P05 transparency log.
- **IPRS fail-closed** — no silent auto-approve on lookup failure.
- **NFC name normalisation** + canary rows on every batch.
- **ABIS soft-hold on proposed removals** — no biometric template hard-deleted without dual confirmation.
- **`*400#` family short-code 90-day grace window** — never a same-day removal.
- **T-60 / T-30 freeze race defence** — OPA hard-close inside a serialisable PG tx (M04).

## 4. Kit + device fraud

- **TDX / SEV-SNP DCAP attestation** (`services/abis-svc/crates/abis-attest`) — 32-byte nonce + 60 s freshness + `UNIQUE (kit_id, nonce)` replay defence.
- **Play Integrity + App Attest + TPM + software-tier** profiles per `eel_kits` (V021).
- **SPIRE SVID per kit** (`spiffe://eel.local/kit/<id>`).
- **Kit certificate revocation ≤60 s SLA** (M06); honey-mode routing for revoked kits.
- **Append-only per-kit hash chain** on `eel_kit_attestation_events`.
- **`X-Eel-Signature` HMAC-SHA256** on every webhook.
- **GNSS + IMU spoof checks** (M05) — multi-constellation + inertial cross-check.
- **Time-fence FSM** — kits refuse operation outside assigned polling window.
- **Dual-control close** — no single officer can close a station.

## 5. Voter fraud, coercion, impersonation

- **1:1 multi-modal biometric verification** (M05 + M06) — fingerprint + face + iris fusion via `abis-fusion-advanced`.
- **s.44A four-eyes complementary mechanism** with IAB 5% Merkle-seed sample (M25).
- **IDP double-cast guard** — serialisable PG tx.
- **REFUGEE_VOTING_ENABLED flag gate** on M25 (default false).
- **REMAND + BARRACKS dual-biometric open** — dual-officer biometric + HSM required.
- **PAD Level 2** across all modalities — iBeta L1 + L2 detectors (fingerprint multispectral + LivDet; face LBP + moire + challenge-response + 3DMM pose consistency; iris pupil dynamics + Purkinje reflex + contact lens; voice replay + synthesis + conversion).
- **Duress finger** — covert coercion signal silent to ECC.
- **Twin registry** — pre-declared groups suppress false-match alarms.
- **Cluster analysis** — nightly union-find over adjudicated matches; catches multi-way collisions.

## 6. Insider fraud (staff, officers, adjudicators)

- **Ed25519 HSM signature** on every reviewer action with `prev_action_hash` chain; append-only trigger blocks UPDATE + DELETE.
- **Tier-3 panel voting** — 3-of-5 quorum; unique constraint prevents double-vote.
- **Reviewer specialisation + max_daily_cases + shift-tracking**.
- **LMS training gate on M33** — worker with no valid M38 certificate cannot be dispatched.
- **Segregation of Duties** at API + DB layers on `PaymentVoucher::approve` and `PayrollRun::approve` (M54 + M55).
- **JIT admin credentials ≤4 h + session recording** via WP-2.3 IAM seed.
- **Session-scoped SpiceDB permission checks** in every human surface.
- **Kyverno + OPA policies** block unsigned images and pods without SVID.

## 7. Transmission / signal-path rigging

- **M62 transmission-legal-basis-audit** — every `TransmissionEvent` gets a `ComplianceVerdict` from a `LegalRule` engine; rule-drift monitor catches silent rule changes.
- **5-channel STH publication** (IEBC web + GitHub mirror + IPFS-KE + Kenya Gazette + KBC Radio) — split-view attack requires all 5 to diverge publicly.
- **Witness cosigning** — IAB day-1; external observers day-2; witness API + cron in `transparency-svc/crates/tlog-witness`.
- **Domain-tagged leaves** — cannot smuggle in a leaf without its context.
- **Split-view detection** — `tlog-witness` cross-checks cosigs and pages within 60 s.
- **SMS + USSD fallback codec** (CBOR + zstd) with monotone counter replay defence.

## 8. Financial fraud

- **M22 campaign-finance-transparency** — Ceiling engine + CashAggregate cross-check with CBK + FRC doubles; SAR triggers referrals to ODPP + DCI.
- **M66 election-financial-forensics** — RD (Imbens-Lemieux 2008), IV (Angrist-Pischke 2009), DiD (Card-Krueger 1994).
- **M47 nomination-fee-payment** — idempotent Daraja C2B with per-TransID dedup; B2C batch idempotency via Originator-Conversation-ID.
- **M59 OAG interface** — scope + evidence + response tracked; scope-creep guards.
- **BagIt v1.0 (RFC 8493) + OAIS ISO 14721** archive with SHA-256 preservation attestation on financial artefacts (M69).

## 9. Coordinated inauthentic behaviour (CIB) + info-ops

- **M17 speech-integrity** — content flags + CIB clustering (community detection on user-behaviour graph) + deepfake detection interface (MesoNet, FaceForensics++, RawNet2) + Kenyan-language bias report.
- **Cross-source flag correlation** — same false claim across multiple platforms flags CIB.
- **M13 public-verifier-app** — clone detection + Play Integrity + App Attest.

## 10. Cryptographic + supply-chain rigging

- **Reproducibility bundle v2** (M14) — bit-identical rebuild across x86_64 + aarch64 + darwin via Nix.
- **Post-quantum hybrid signatures** — STHs signed with ML-DSA + ECDSA P-384; TLS 1.3 with ML-KEM + X25519.
- **cosign + SLSA L3 provenance** on every container image.
- **Nexus-mirrored dependencies** with pinning policy.
- **RFC 3161 TSA timestamps** on every archived artefact.
- **HSM M-of-N key ceremony** (3-of-5) for KEK generation; break-glass Shamir shares.

## 11. Post-election audit + petition

- **M21 rla-audit-tool** — SHANGRLA risk-limiting audit (Stark 2020) with NIST beacon + HSM + IAB smartcard combined entropy; Tauri offline desktop; T-statistic auto-recount trigger.
- **M40 election-court-it-interface** — 15-item evidence bundles with petitioner access + supplementary requests + technical witnesses; eCourt integration.
- **M41 post-election-review** — findings + recommendations + parliament referrals + long-drip silence alerts + public tracker.
- **M62 transmission-legal-basis-audit** — every event has a compliance verdict; party portal shows filtered view.
- **M61 secrecy-assurance-testing** — red-team engagements on isolated staging clones with IAB attestation.

## 12. Chaos + resilience testing

- `chaos/scenarios/` — 34 experiments including HSM loss, split-view, mass revocation, sequencer stall, tamper-topic flooding, canary-token trigger, SPIRE + SpiceDB kill.
- `verification/security` — 24-hour fuzz across 27 boundaries (envelope, RFC 8785, ML-DSA, Merkle proof, CBOR, zstd, MRZ, SMS/USSD, QR badge, shapefile, PDF/image ICR, PKCS#11, attestation tokens, SAML/OIDC, timestamp, archive extractor).
- `platform/observability/alerts/` per-service SLO burn alerts.

## 13. 24 hardening recommendations

### High impact, moderate effort

1. **Real trained M26 anomaly models** on Kenyan historical elections (2013 + 2017 + 2022).
2. **Zero-knowledge proof of correct tally** — Groth16 / Bulletproofs SNARK; new `abis-zk-tally` crate.
3. **Cross-vendor ABIS consensus at peak** (T-1..T+7) — enable dual-vendor 2-of-3 vote via `abis-fp-consensus`.
4. **Public reproduce challenge** — nightly bounty invites independent teams to submit their own reproduce output; automatic diff.
5. **Post-quantum signature on every leaf**, not just STHs — extend eel-events envelope; ~2x storage.
6. **UEBA layer on Falco + Tetragon** — user + entity behaviour analytics for compromised staff accounts.
7. **Adversarial-training on speech-integrity + deepfake models** — ASVspoof 2021 + FaceForensics++ + DFDC corpora.

### High impact, high effort

8. **BFT ordering for tally aggregation** — HotStuff or Tendermint replacing Kafka-transactional; adds latency but eliminates single-point-of-rewrite.
9. **Homomorphic tally over encrypted 34A leaves** — CKKS on ciphertexts; only decryption public.
10. **Cross-platform CIB correlation** — GNN across Facebook + Twitter + TikTok + WhatsApp broadcast.
11. **Encrypted-inputs adjudication** — shielded-only review for low-tier cases via `abis-shielding`.
12. **Public-key transparency for kit certificate issuance** — CT-log-style for kit cert issuance.

### Medium impact, low effort

13. **Live IAB witness runbook + smartcard ceremony** for T-1..T+7.
14. **Behavioural biometrics on RO desktop sessions** — keystroke + mouse trajectory on M09.
15. **Threat-intel STIX 2.1 feed integration** in ECC (M16).
16. **Cryptographic binding of physical seal photo** — extend M20 custody events.
17. **Multi-party 4-of-7 KEK rotation ceremony** during freeze window.
18. **TLA+ TLC model-check in nightly CI**.
19. **Independent Assurance Group (IAG)** — separate from IAB, runs reproduce every 24 h during election week.
20. **Sample-based parallel manual count** — 1% of stations at T+3; feed RLA; publish divergence.

### Housekeeping

21. **Cross-service audit-shadow prev_hash chain** — extend adjudicator pattern to every service.
22. **Ensemble voting on every AI model** — K-of-N independent models must agree before auto-action.
23. **Model-canary auto-rollback on any fairness slice breach**.
24. **PII-in-events Semgrep rule** — fail PR if a Tier-A/B field lands in an unallowed event schema.

## 14. Implementation status

All 24 recommendations are being implemented in follow-up work packages:

| WP | Group | Items |
|---|---|---|
| HARD-A | Crypto + ML | 1 (real M26 training harness), 2 (ZK tally), 5 (PQ per-leaf), 7 (adversarial training) |
| HARD-B | Consensus + ordering | 3 (cross-vendor ABIS), 8 (BFT ordering), 9 (FHE tally) |
| HARD-C | Ops + governance | 4 (public reproduce challenge), 6 (UEBA), 13 (IAB runbook), 14 (behavioural biometrics), 15 (STIX feed), 17 (4-of-7 KEK), 20 (parallel manual count) |
| HARD-D | CIB + adjudication + edge | 10 (cross-platform CIB), 11 (encrypted adjudication), 12 (kit cert transparency), 19 (IAG governance) |
| HARD-E | Housekeeping | 16 (physical seal binding), 18 (TLC in CI), 21 (cross-service audit chain), 22 (ensemble voting), 23 (canary fairness rollback), 24 (PII Semgrep) |

Each WP is landing a new crate or extending an existing one under `services/*/crates/hardening-*` with tests, plus documentation updates here. Completion status will be tracked in `docs/audit/hardening-status.md`.
