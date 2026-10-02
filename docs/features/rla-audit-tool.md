# Risk-limiting audit tool (M21)

Module M21 (`rla-audit-tool`) owned by `governance-svc`. Implements a SHANGRLA risk-limiting audit (Stark 2020, Journal of the American Statistical Association) across tabulated results, driven by five entropy sources XOR-combined with a NIST Randomness Beacon seed anchored under an IAB HSM card. A T-statistic threshold auto-triggers a full recount when the risk limit is not met. Auditors work offline on a signed Tauri desktop app.

## Contents

1. What the RLA audit tool does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What the RLA audit tool does (vs related modules)

- **M09 Results tabulation** = aggregates and certifies contest totals from Form 34 series; M21 consumes those certified totals as its audit input.
- **M21** = post-tabulation statistical verification: sample a subset of physical ballots using cryptographically auditable randomness, compare manually-counted tally against the tabulated result, compute a risk bound, and determine whether the result meets the statutory risk limit or requires a full recount.
- **M26 Anomaly detection** = algorithmic fraud detection on telemetry and result distributions; M21 provides a statistically principled complement focused on ballot-level physical verification.

No M21 output modifies a certified result directly. The only outcome is PASSED (risk limit met), ESCALATED (full recount required), or INSUFFICIENT_BALLOTS (physical ballot audit trail incomplete).

## 2. Data plane

Five aggregates under `services/governance-svc/crates/rla-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **AuditRound** | RLA session: contest_id, risk_limit (%), seed, entropy_sources (JSON), status, started_at, completed_at. | A |
| **SampleBallot** | Selected ballot for physical inspection: ballot_id, polling_station_id, batch_seq, stratum, status (PENDING/COUNTED/DISCREPANT). | A |
| **TallyReport** | Manual count for a sample ballot: auditor_id, candidate_tallies (JSON), discrepant (bool), sig (auditor Ed25519). | A |
| **RiskBound** | Running SHANGRLA T-statistic per contest: T_value, risk_limit, round_number, auto_recount_triggered (bool). | A |
| **FullRecount** | Auto-triggered recount order: contest_id, reason, chair_sig (HSM), routed_to (Judiciary / the Commission). | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Certified tabulation totals (M09) | Per contest certification | Internal Kafka event | SVID mTLS | Audit blocked until M09 certification event received |
| NIST Randomness Beacon (beacon.nist.gov) | Per round | REST/JSON | TLS (public) | Circuit breaker; fallback to local pseudo-random with warning; beacon gap logged |
| IAB HSM seed (card-based) | Per round | Physical token + HSM API | HSM device cert | HSM failure blocks round start; failsafe requires IAB chair approval |
| Physical ballot count (auditor tablet) | Per sample | Signed form submission | JWT (auditor device cert) | Offline Tauri form; synced when connectivity restored |
| Entropy sources 3-5 | Per round | Real-time feeds (NTP stratum, OS entropy pool, M09 hash chain) | Internal | XOR combined with NIST + IAB; one source failure reduces confidence; two source failures block round |

## 4. Pipeline

```
M09 certified totals received
  -> AuditRound created with risk_limit from Elections Act s.83
  -> seed generation:
       source1 = NIST Randomness Beacon pulse_value
       source2 = IAB HSM card Ed25519 signature of round_id
       source3 = NTP stratum-1 timestamp hash
       source4 = OS /dev/urandom 256-bit draw
       source5 = SHA-256(M09 certified_result_root)
       seed = source1 XOR source2 XOR source3 XOR source4 XOR source5
  -> SHANGRLA sample size computed per contest:
       T-statistic target given risk_limit (default 5%)
       stratified by polling station (Stark 2020 COBRA extension)
  -> SampleBallot list generated deterministically from seed
  -> auditor notification (offline Tauri desktop app syncs list)
  -> auditor physically locates ballots; enters TallyReport
  -> per-ballot TallyReport received:
       SHANGRLA T-statistic updated (running product of likelihood ratios)
       RiskBound record updated
       if T >= 1/risk_limit: status = PASSED
       if sample exhausted and T < 1/risk_limit:
         auto_recount_triggered = true
         FullRecount created; Chair HSM signature required;
         eel.rla.full_recount_ordered.v1 emitted
  -> eel.rla.round_completed.v1 anchored in P05 Merkle log
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/rla.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /rla/v1/rounds` | IAB officer (JWT + HSM) | Initiate audit round; seed generated server-side |
| `GET /rla/v1/rounds/{id}` | IAB auditor / public | Round status, sample size, T-statistic |
| `GET /rla/v1/rounds/{id}/ballots` | IAB auditor (JWT) | Sample ballot list with location info |
| `POST /rla/v1/rounds/{id}/tallies` | IAB auditor (JWT, device cert) | Submit TallyReport for a sample ballot |
| `GET /rla/v1/rounds/{id}/risk` | IAB officer / public | Running risk bound; T-statistic |
| `POST /rla/v1/rounds/{id}/recount` | the Commission Chair (JWT + HSM) | Sign and route FullRecount order |
| `GET /rla/v1/seed/{round_id}/entropy` | IAB auditor / public | Full entropy source disclosure for seed audit |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Auditor Tauri desktop app | `services/governance-svc/tauri/rla-auditor/` | Offline-capable; syncs sample list; auditor enters tally; signed submission |
| RLA dashboard (Phoenix LiveView) | `services/governance-svc/ex/rla_web/` | IAB chair monitors T-statistic, sample progress, entropy source health |
| Public results page (Astro) | `services/governance-svc/astro/rla-public/` | T-statistic per contest, seed disclosure, round outcome visible to public |
| Admin panel (Django) | `services/governance-svc/django/rla-admin/` | IAB admin manages auditor accounts, recount routing |

WCAG 2.2 AA on all surfaces. The Tauri app runs on Windows and macOS; Linux not required. Languages: EN + SW.

## 7. Integrity + anti-tamper

- **SHANGRLA T-statistic** (Stark 2020) -- the running product of per-ballot likelihood ratios provides a mathematically rigorous risk bound; the T-statistic value and formula are stored with every `RiskBound` update so any third party can recompute from the published tally reports.
- **Five-source XOR seed** -- no single party controls the seed; IAB HSM card controls only one of five sources; NIST Beacon provides public verifiability; OS entropy and NTP prevent any single external party from predicting the sample.
- **Deterministic sample generation** -- given the published seed and the certified ballot manifest, any party can independently recompute the sample list; the Tauri app ships with the sample generation algorithm and the published manifest.
- **Auditor Ed25519 signatures** -- every `TallyReport` is signed by the auditor's device cert; unsigned reports are rejected; the auditor public key is registered in the IAB key store before the round starts.
- **Auto-recount trigger** -- the T-statistic threshold is computed from the risk limit; the system cannot certify a result that has not met the limit; human sign-off (Chair HSM) is required to route the recount order but not to block the election result from being challenged.
- **P05 Merkle anchor** -- round completion event anchored in the transparency log; courts can verify the audit independently of the governance-svc database.

## 8. Kenya statute alignment

- **Elections Act s.83 (statutory risk limit)** -- risk limit value configurable but defaults enforced from s.83; any deviation from default requires Chair HSM + IAB commissioner cosignature and is logged.
- **Elections (Technology) Regulations 2017** -- audit trail retention for 7 years; every TallyReport signed and Merkle-anchored.
- **the electoral authority's establishing Act s.7** -- IAB independence guaranteed by the dual-key seed design; the Commission cannot unilaterally determine the sample.
- **Constitution of Kenya 2010 Art. 86(c)** -- results accuracy and verifiability; SHANGRLA provides quantitative risk bound satisfying the verifiability obligation.
- **DPA 2019 s.26** -- ballot physical identifiers are not retained in the digital store beyond the audit round; only ballot sequence numbers and batch references stored.

## 9. Honest caveats + external prerequisites

- SHANGRLA assumes the physical ballot manifest is complete and accurate. If ballots are missing from the manifest (M09 ingestion failure, lost custody chain from M20), the audit produces INSUFFICIENT_BALLOTS rather than PASSED or ESCALATED; this is logged but cannot be automatically resolved.
- The NIST Randomness Beacon is a US government service; network access must be confirmed from the audit environment. The circuit breaker fallback to local pseudo-random reduces verifiability; any such event must be publicly disclosed.
- The Tauri desktop app requires Windows 10+ or macOS 13+; auditor device procurement must confirm OS compatibility.
- The physical ballot retrieval workflow (matching the SampleBallot list to physical ballot boxes) depends on M20 custody chain completeness; an incomplete M20 chain means auditors cannot locate the sampled ballot and it is marked DISCREPANT.
- The statutory risk limit value in Elections Act s.83 must be confirmed with the Commission legal before the first live round; the spec defaults to 5% but the Act's precise wording is interpreted differently by some practitioners.

## 10. Test coverage + operational runbooks

Coverage under `services/governance-svc/crates/rla-api/tests/` and the Tauri app's test suite: unit tests for SHANGRLA T-statistic computation, seed XOR combiner, NIST Beacon parser, sample generator, and auto-recount trigger; property-based tests (proptest) asserting T-statistic monotonicity for any sequence of concordant ballots; integration tests with SoftHSM IAB card, NIST Beacon stub, M09 certification event double, and P05 tlog stub; e2e scenario covering round initiation through sample generation, tally submission, T-statistic update, and PASSED outcome; chaos scenario where NIST Beacon is unavailable (verifies circuit-breaker fallback and warning log); Tauri app e2e using WebDriver (tauri-driver) for offline sync and tally entry.

Runbooks under `docs/runbooks/`:

- `rla-degradation.md` (create): NIST Beacon unavailable; IAB HSM fail; Tauri app offline sync failure; recount routing failure.
- `full-recount-procedure.md` (create): step-by-step for Chair HSM signing and judiciary routing.

Dashboards: `platform/observability/governance-svc/grafana-rla.json` (T-statistic per contest, sample completion rate, entropy source health, recount count). SLO alerts: T-statistic computation lag >60 s; entropy source count <3; recount order unsigned >30 min.

Chaos scenarios under `chaos/scenarios/eel-rla-*.yaml`: NIST beacon unavailable (triggers HSM TRNG fallback); auditor Tauri desktop offline sync failure; T-statistic computation lag under high tally-report volume.

Performance targets: T-statistic update < 2 s per TallyReport submission; sample list generation < 10 s for 100 000 ballots at p99; P05 round anchor lag < 60 s.

Seed verification: the published seed (five XOR sources) and ballot manifest are sufficient for any third party to independently recompute the sample list and verify that M21's sample was not cherry-picked. The Tauri desktop app ships with the sample-generation algorithm and the signed manifest for exactly this purpose.

SHANGRLA risk limit default 5% per Elections Act s.83. Any change to the risk limit requires the Commission Commissioner resolution and a new HSM-signed AuditRound configuration record before the next round opens.

Auditor training requirement: each auditor must complete the RLA simulation in the Tauri desktop app against a synthetic election before being assigned to a live round; completion is recorded in the `AuditRound.auditor_roster` and the IAB chair must countersign the roster before the round opens.

Related: `docs/plans/modules/rla-audit-tool.md`; `services/governance-svc/docs/CAPABILITIES.md`; edge-case matrix EC-RLA-* in `docs/plans/08-edge-case-matrix.md`.
