# Election financial forensics (M66)

Module M66 (`election-financial-forensics`) owned by `governance-svc`. Performs post-election financial forensics using regression discontinuity (RD), instrumental variables (IV), and difference-in-differences (DiD) estimators to detect anomalous candidate spending patterns. Produces `ReferralPacket` evidence bundles for the Financial Reporting Centre (FRC) and coordinates with KNBS on census-calibration of baseline spending estimates.

## Contents

1. What Election Financial Forensics does (vs M22 Campaign Finance Transparency)
2. Data plane (schema)
3. Estimation methods
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Election Financial Forensics does (vs M22 Campaign Finance Transparency)

- **M22** (Campaign Finance Transparency) = statutory disclosure: candidates submit contribution and expenditure returns; these are published for public scrutiny.
- **M66** = forensic analysis layer: takes M22 disclosures plus auxiliary data (M10 results, KNBS socioeconomic datasets) and runs causal-inference estimators to flag statistically anomalous spending-to-votes relationships.

M22 surfaces what was declared. M66 asks whether the declared spending pattern is consistent with the observed vote distribution, using causal estimators that are robust to confounding (Imbens and Lemieux, Journal of Econometrics 2008 on RD; Angrist and Pischke, Mostly Harmless Econometrics, Princeton University Press 2009 on IV and DiD).

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/forensics/src/models.rs` and the M66 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **AnalysisCycle** | One forensic analysis run: `cycle_id`, `election_cycle_id`, `estimator_versions` (JSONB: RD version, IV version, DiD version), `knbs_dataset_version`, `started_at`, `completed_at`, `status` (RUNNING / COMPLETE / FAILED). | A |
| **EstimateRecord** | Output of one estimator for one candidate: `estimate_id`, `cycle_id`, `candidate_id`, `constituency_code`, `estimator_type` (RD / IV / DiD), `point_estimate`, `confidence_interval_95` (lower, upper), `p_value`, `anomaly_flag` (bool, true if p < 0.05 and magnitude exceeds threshold), `estimator_params_json`. | A |
| **ReferralPacket** | Evidence bundle for FRC: `packet_id`, `cycle_id`, `candidate_ids` (array), `estimate_ids` (array), `narrative_md`, `replication_package_url` (MinIO), `submitted_at`, `frc_ref`, commissioner HSM signature. | A |
| **ReplicationPackage** | Reproducible analysis archive: `package_id`, `cycle_id`, `code_commit_sha`, `data_snapshot_urls` (JSONB: MinIO paths for each input dataset), `environment_lockfile_url`, `results_csv_url`, `package_hash` (SHA-256). | A |

## 3. Estimation methods

All estimators are implemented in `services/governance-svc/crates/forensics/src/estimators/`:

- **Regression discontinuity** (Imbens and Lemieux, Journal of Econometrics 2008; Lee and Lemieux, Journal of Economic Literature 2010): detects spending anomalies near electoral thresholds. Implementation uses the `rdrobust` algorithm (Calonico, Cattaneo, and Titiunik, Econometrica 2014) with MSE-optimal bandwidth selection.

- **Instrumental variables** (Angrist and Pischke, Princeton University Press 2009): uses KNBS census cell as an instrument for baseline spending to identify exogenous spending variation. Two-stage least squares (2SLS) estimated via the `linfa-linear` crate.

- **Difference-in-differences** (Bertrand, Duflo, and Mullainathan, Quarterly Journal of Economics 2004): compares spending-to-votes ratios across constituencies treated and untreated by a specific campaign event or regulatory change, with clustered standard errors at the county level.

Anomaly threshold: `anomaly_flag = true` when `p_value < 0.05` AND `|point_estimate| > 2 * median(point_estimates for cycle)`. Both conditions must hold to reduce false positive referrals.

## 4. Pipeline

Files on disk under `services/governance-svc/crates/forensics/src`: `lib.rs`, `models.rs`, `estimators/rd.rs`, `estimators/iv.rs`, `estimators/did.rs`, `referral.rs`, `replication.rs`, `routes/`.

```
post-election trigger (eel.rts.results_certified.v1 from M51, T+30)
      -> forensics officer creates AnalysisCycle
      -> input data assembled:
            -> M22 disclosures (campaign_finance_disclosures table)
            -> M10 results (constituency vote totals)
            -> KNBS census dataset (versioned CSV from KNBS API)
      -> ReplicationPackage assembled:
            -> code commit SHA pinned; environment lockfile generated
            -> input data snapshots uploaded to MinIO
            -> package_hash computed
      -> estimators run in parallel (tokio::task::spawn_blocking):
            -> RD estimator: per-constituency, per-candidate
            -> IV estimator: county-level panel
            -> DiD estimator: constituency pairs
      -> EstimateRecord created for each output
      -> anomaly_flag applied per threshold rules
      -> AnalysisCycle status = COMPLETE; emit eel.governance.forensics_complete.v1
      -> if any anomaly_flag == true:
            -> forensics officer drafts ReferralPacket
            -> narrative_md compiled; commissioner signs with HSM
            -> packet submitted to FRC; frc_ref stored
            -> emit eel.governance.referral_submitted.v1
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/forensics.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/forensics/cycles` | Forensics officer (JWT + SpiceDB `FORENSICS_OFFICER`) | Create analysis cycle |
| `GET /v1/forensics/cycles/{id}` | Forensics officer | Cycle status + estimator run progress |
| `GET /v1/forensics/cycles/{id}/estimates` | Forensics officer, OAG auditor | All estimate records for cycle |
| `GET /v1/forensics/cycles/{id}/estimates?anomaly=true` | Forensics officer | Flagged anomalies only |
| `POST /v1/forensics/cycles/{id}/referrals` | Forensics officer | Draft referral packet |
| `PATCH /v1/forensics/cycles/{id}/referrals/{rid}/sign` | Commissioner + HSM | Sign referral packet |
| `PATCH /v1/forensics/cycles/{id}/referrals/{rid}/submit` | Forensics officer | Submit to FRC |
| `GET /v1/forensics/cycles/{id}/replication-package` | Public (rate-limited 10/hr) | Download replication package |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Forensics analysis console (Django) | `services/governance-svc/django/forensics_admin` | Create cycles, review estimates, draft referrals |
| Anomaly review workspace (LiveView) | `services/governance-svc/ex/forensics_web` | Side-by-side view of estimate, M22 disclosure, M10 results |

Languages: EN. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Replication package**: the `ReplicationPackage.package_hash` covers the code, data, and environment lockfile; any third party can download the package and reproduce the estimates to verify the `anomaly_flag` decisions.
- **HSM-signed referral**: commissioner HSM signature over `SHA-256(packet_id || narrative_md || estimate_ids)` ensures referral packets cannot be altered after submission to FRC.
- **Estimator version pinning**: `AnalysisCycle.estimator_versions` records the exact crate versions used; this ensures historical cycles can be replicated even after estimator updates.
- **Anomaly threshold versioning**: the anomaly threshold rules are versioned in `services/governance-svc/config/forensics_thresholds.toml`; changes to the threshold require a new `AnalysisCycle` and do not retroactively change past `EstimateRecord.anomaly_flag` values.
- **Audit shadow**: all cycle, estimate, and referral mutations logged to `eel_governance_audit_shadow`.

## 8. Kenya statute alignment

- **Election Campaign Financing Act 2013 §23 (enforcement)**: the FRC is the statutory body for enforcing campaign finance limits; `ReferralPacket` is the formal mechanism for the Commission to refer anomalous candidates to the FRC.
- **Election Campaign Financing Act 2013 §6 (disclosure)**: M22 disclosures (the input to M66) are the statutory returns that candidates must file.
- **Kenya Data Protection Act §26 (minimisation)**: candidate-level estimates use constituency-level aggregates from M10, not individual voter data; no personal data about voters enters the forensics pipeline.
- **Statistics Act 2006 §23 (KNBS data sharing)**: KNBS census data used as IV baseline requires a data-sharing agreement with KNBS; M66 records the `knbs_dataset_version` to prove which agreed dataset was used.

## 9. Honest caveats + external prerequisites

- The forensic estimators detect statistical anomalies; they do not prove fraud. A referral to FRC is a flag for investigation, not a finding of guilt. The `narrative_md` must clearly state this distinction.
- KNBS API integration requires a data-sharing agreement. If the KNBS API is unavailable, the forensics officer must upload the census dataset manually; the `ReplicationPackage` records the upload path.
- RD bandwidth selection (Calonico, Cattaneo, and Titiunik MSE-optimal) requires a minimum of 30 observations near the threshold; constituencies with fewer than 30 candidates near a threshold will have no RD estimate. These are recorded as `EstimateRecord` rows with `p_value = NULL` and `anomaly_flag = false`.
- The estimators are written in Rust using `linfa`; statistical output has been validated against R `rdrobust` and `AER` packages on synthetic data. Production validation against real election data requires a calibration run with KNBS and FRC before go-live.
- The replication package is publicly downloadable (rate-limited); sensitive candidate financial details visible in M22 disclosures are excluded from the replication package; only aggregated constituency-level inputs are included.

## 10. Test coverage + operational runbooks

Coverage under `services/governance-svc/crates/forensics/tests/`: RD estimator (property-based, validation against R rdrobust on synthetic data), IV 2SLS (property-based, validation against R AER), DiD clustered SE (property-based), anomaly flag threshold (table-driven), replication package hash (property-based), referral packet HSM signature (integration).

Runbooks under `docs/runbooks/`:

- `governance-svc.md` (section: election-financial-forensics) -- deploy, rollback, estimator version update.
- `frc-referral-submission.md` -- manual FRC submission when API is unavailable.
- `knbs-data-unavailable.md` -- manual census data upload procedure.

Related documents: full spec at `docs/plans/modules/election-financial-forensics.md`; service capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`; M22 at `docs/features/campaign-finance-transparency.md`.

### Estimator input and output summary

| Estimator | Unit of analysis | Key input | Key output |
|---|---|---|---|
| RD (Calonico et al., Econometrica 2014) | Candidate at threshold | Spending near margin cutoff, vote share near cutoff | Local average treatment effect at cutoff; optimal bandwidth |
| IV 2SLS (Angrist and Pischke, 2009) | Constituency | KNBS census cell as instrument, declared spending | Exogenous spending effect on vote share |
| DiD (Bertrand, Duflo, Mullainathan, QJE 2004) | Constituency pair | Pre/post campaign event, treated/untreated split | Differential spending-to-vote change; clustered SEs |

All three estimators are run per `AnalysisCycle`; a candidate is flagged (`anomaly_flag = true`) only when at least two of the three estimators independently show `p_value < 0.05` and the magnitude condition. This reduces false positives relative to single-estimator flagging.

### Key forensics SLOs

| Metric | Target |
|---|---|
| Analysis cycle completion time (250 candidates) | <= 4 h |
| Replication package generation time | <= 30 min |
| FRC referral packet submission lead time | >= 7 days before FRC deadline |
