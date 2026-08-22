# Parliamentary and judiciary reporting (M58)

Module M58 (`parliamentary-judiciary-reporting`) owned by `governance-svc`. Manages statutory reports to Parliament (select committees, Speaker notifications), testimony preparation and logging, judiciary handover packages for election petitions, and Supreme Court pre-registration of certified result materials.

## Contents

1. What Parliamentary and Judiciary Reporting does (vs M40 Election Court IT Interface)
2. Data plane (schema)
3. External integrations
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Parliamentary and Judiciary Reporting does (vs M40 Election Court IT Interface)

- **M40** (Election Court IT Interface) = petition evidence submission to the High Court and Supreme Court after an election petition has been filed. M40 is reactive: it responds to court orders.
- **M58** = proactive statutory reporting: submits periodic and event-triggered reports to Parliament, prepares and logs IEBC officer testimony, and pre-registers certified results and Merkle proofs with the Supreme Court registry before any petition.

M58 and M40 share the `StatutoryReport` and `JudiciaryHandover` aggregates but write them for different triggers. M40 is petition-driven; M58 is schedule-driven and pre-registration-driven.

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/parl-judiciary/src/models.rs` and the M58 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **StatutoryReport** | Report to Parliament: `report_id`, `committee` (LEGAL_AFFAIRS / BUDGET / SECURITY / JOINT_SELECT), `report_type` (ANNUAL / POST_ELECTION / INCIDENT / SUPPLEMENTARY), `period_start`, `period_end`, `body_md`, `tabled_at`, `parliamentary_ref`, commissioner HSM signature. | A |
| **Testimony** | Officer testimony record: `testimony_id`, `officer_id`, `forum` (PARLIAMENT / HIGH_COURT / SUPREME_COURT / COMMITTEE), `scheduled_at`, `conducted_at`, `summary_md`, `recording_url` (MinIO), `reviewed_by_legal`. | B |
| **JudiciaryHandover** | Evidence handover package to court: `handover_id`, `court` (HIGH_COURT / SUPREME_COURT), `petition_ref` (may be null for pre-registration), `content_type` (CERTIFIED_RESULTS / MERKLE_PROOF / TRANSMISSION_LOG / AUDIT_TRAIL), `package_hash` (SHA-256), `received_by_registrar_at`, `registrar_ref`. | A |
| **SCPreRegistration** | Supreme Court pre-registration: `prereg_id`, `election_cycle_id`, `certified_results_handover_id`, `merkle_proof_handover_id`, `registered_at`, `sc_registry_ref`. | A |

## 3. External integrations

| Integration | Direction | Auth | SLA | Failure handling |
|---|---|---|---|---|
| **Parliamentary portal** (National Assembly + Senate) | Outbound (statutory report tabling) | Parliamentary API key | Per parliamentary calendar | Retry; alert legal officer 14 days before tabling deadline |
| **Supreme Court registry** | Outbound (pre-registration, JudiciaryHandover) | Registry API + physical courier confirmation | Within 7 days of result certification | Retry API; physical courier as fallback; alert chief registrar |
| **High Court e-filing** | Outbound (petition handovers, M40 integration) | High Court API key | Per court order deadline | Retry; escalate to M40 if court order imminent |
| **MinIO** (testimony recordings) | Internal | S3-compatible IAM | Immediate | Hard fail: testimony record incomplete without MinIO URL |

## 4. Pipeline

Files on disk under `services/governance-svc/crates/parl-judiciary/src`: `lib.rs`, `models.rs`, `statutory_report.rs`, `testimony.rs`, `judiciary_handover.rs`, `sc_preregistration.rs`, `routes/`.

```
election cycle certified (eel.rts.results_certified.v1 from M51)
      -> M58 triggers SCPreRegistration workflow:
            -> assemble certified results package (from M51 CertificateOfElection)
            -> assemble Merkle proof package (from P05 transparency log)
            -> compute package_hash (SHA-256) for each package
            -> create JudiciaryHandover records for SC
            -> submit to SC registry API
            -> log registrar_ref in SCPreRegistration
            -> emit eel.governance.sc_preregistration_complete.v1

parliamentary calendar event (cron or manual trigger)
      -> legal officer drafts StatutoryReport
            -> body_md compiled; commissioner reviews
            -> commissioner signs with HSM
      -> submitted to parliamentary portal
            -> parliamentary_ref stored
            -> emit eel.governance.statutory_report_tabled.v1

officer summoned to testify
      -> legal officer creates Testimony record (forum + scheduled_at)
      -> officer prepares; legal review of summary_md
      -> testimony conducted; recording uploaded to MinIO
      -> summary finalised; emit eel.governance.testimony_recorded.v1

petition filed (M40 triggers JudiciaryHandover)
      -> legal officer assembles evidence package per court order
      -> package_hash computed; submitted to court via M40 e-filing
      -> registrar_ref received and stored
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/parl-judiciary.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/parl-judiciary/statutory-reports` | Legal officer (JWT + SpiceDB `LEGAL_OFFICER`) | Draft statutory report |
| `PATCH /v1/parl-judiciary/statutory-reports/{id}/sign` | Commissioner + HSM | Sign report |
| `PATCH /v1/parl-judiciary/statutory-reports/{id}/table` | Legal officer | Submit to parliamentary portal |
| `POST /v1/parl-judiciary/testimony` | Legal officer | Create testimony record |
| `PATCH /v1/parl-judiciary/testimony/{id}/record` | Legal officer | Finalise testimony + upload recording |
| `POST /v1/parl-judiciary/handovers` | Legal officer | Create judiciary handover package |
| `GET /v1/parl-judiciary/handovers/{id}/hash` | Court registry (SVID) | Verify package hash |
| `POST /v1/parl-judiciary/sc-preregistration` | System (triggered by M51 event) | Initiate SC pre-registration |
| `GET /v1/parl-judiciary/sc-preregistration/{cycle_id}` | Commissioner + legal officer | Pre-registration status |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Legal officer console (Django) | `services/governance-svc/django/parl_judiciary_admin` | Draft reports, manage testimony, track handovers |
| SC pre-registration dashboard (LiveView) | `services/governance-svc/ex/sc_prereg_web` | Monitor pre-registration completion in real time post-certification |

Languages: EN (all statutory documents). WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **HSM-signed statutory reports**: commissioner HSM signature covers `SHA-256(report_id || body_md || tabled_at)`; any post-signature edit to the body is detectable.
- **Package hash verification**: `JudiciaryHandover.package_hash` is SHA-256 of the serialised evidence package; the court registry's `GET /hash` endpoint allows a registrar to verify the package has not been altered since submission.
- **SC pre-registration idempotency**: `SCPreRegistration` has a unique constraint on `election_cycle_id`; the pipeline is idempotent against duplicate `eel.rts.results_certified.v1` events.
- **Testimony recording integrity**: MinIO stores the recording with a server-side checksum; the `recording_url` includes the ETag for client-side verification.
- **Audit shadow**: all aggregate mutations logged to `eel_governance_audit_shadow`.

## 8. Kenya statute alignment

- **Elections Act §74 (IEBC annual report to Parliament)**: `StatutoryReport` with `report_type = ANNUAL` tabled within 90 days of financial year end; the pipeline has a cron trigger for this.
- **Constitution of Kenya 2010 Art 163(8) (SC jurisdiction over presidential election)**: `SCPreRegistration` satisfies IEBC's obligation to provide certified results to the SC before petition window closes (14 days post-declaration, Art 140(1)).
- **Evidence Act Cap 80 §78 (electronic evidence admissibility)**: `JudiciaryHandover.package_hash` is the hook for admissibility: the court can verify the hash against the registry submission to establish authenticity.
- **Parliamentary Standing Orders SO 210 (committee reports)**: `StatutoryReport.committee` taxonomy maps to the official committee names in the 13th Parliament standing orders.

## 9. Honest caveats + external prerequisites

- Parliamentary portal API integration depends on National Assembly and Senate ICT departments providing stable API credentials. Where the portal lacks an API, report tabling falls to manual PDF delivery with `parliamentary_ref` recorded manually by the legal officer.
- SC pre-registration assumes that `eel.rts.results_certified.v1` from M51 fires within 24 hours of the last county tally. If certification is delayed (e.g., prolonged dispute at tallying centre), the pre-registration timeline compresses against the 14-day petition window.
- Testimony recordings are stored in MinIO under the `iebc-testimony` bucket with a 10-year retention policy aligned to the Elections Act records retention schedule. Actual bucket creation and retention policy activation is an operational task at first deployment.
- High Court e-filing integration uses the High Court's online filing system API. The API version is pinned; updates to the court's system require adapter updates.

## 10. Test coverage + operational runbooks

Coverage under `services/governance-svc/crates/parl-judiciary/tests/`: HSM signature verification (integration), package hash round-trip (property-based), SC pre-registration idempotency (integration, duplicate event test), testimony recording ETag (integration with MinIO testcontainer), statutory report tabling (pytest-httpserver mock of parliamentary portal).

Runbooks under `docs/runbooks/`:

- `governance-svc.md` (section: parliamentary-judiciary-reporting) -- deploy, rollback.
- `sc-preregistration-failure.md` -- manual SC registry submission when API is unavailable.
- `testimony-recording-minio-failure.md` -- fallback for recording upload failure.

Related documents: full spec at `docs/plans/modules/parliamentary-judiciary-reporting.md`; service capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`; M40 at `docs/features/election-court-it-interface.md`; M51 at `docs/features/result-declaration-certification.md`.

### Statutory deadlines

| Report or action | Deadline | Alert threshold |
|---|---|---|
| Annual report to Parliament | 90 days after financial year end | T-14 days: warning; T-7: escalation |
| SC pre-registration (presidential election) | T+7 days after result declaration | T+5: warning; T+6: escalation |
| Testimony preparation (officer notice) | 14 days before scheduled date | < 7 days notice: immediate escalation |
| Petition evidence handover | Per court order deadline | T-2 days: warning |

### JudiciaryHandover content types

| Content type | Source module | Typical size |
|---|---|---|
| CERTIFIED_RESULTS | M51 | ~ 500 MB per election cycle |
| MERKLE_PROOF | P05 transparency log | ~ 50 MB |
| TRANSMISSION_LOG | M10 | ~ 2 GB |
| AUDIT_TRAIL | `eel_bvr_audit_shadow`, `eel_rts_audit_shadow` | ~ 5 GB |

Packages are compressed (zstd, Facebook 2015) before hashing; the hash covers the compressed archive, not the uncompressed content.
