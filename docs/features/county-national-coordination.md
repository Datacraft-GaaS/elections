# County and national coordination (M56)

Module M56 (`county-national-coordination`) owned by `governance-svc`. Manages Memoranda of Understanding (MoUs) with county governments, county-level dataset approvals, Ministry of Interior coordination, and cross-agency incidents. Federates identities across SAML-connected county IdPs and escalates critical alerts to the Election Command Centre (M16).

## Contents

1. What County and National Coordination does (vs M16 ECC)
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

## 1. What County and National Coordination does (vs M16 ECC)

- **M16** (Election Command Centre) = situational awareness and incident operations dashboard for IEBC leadership; it consumes escalated alerts.
- **M56** = the coordination layer that creates and tracks the MoUs, dataset handshakes, ministerial approvals, and incidents that feed into M16.

M56 produces the governance events (`eel.governance.critical_incident.v1`) that M16 renders on its map. M16 does not store MoUs or county dataset records; M56 does.

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/coordination/src/models.rs` and the M56 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **MoU** | Memorandum of Understanding with a county government or ministry: `mou_id`, `counterparty_name`, `counterparty_type` (COUNTY / MINISTRY / AGENCY), `signed_at`, `expiry_date`, `scope_summary`, `document_url`, commissioner + counterparty HSM or wet-ink scan digest. | A |
| **CountyDataset** | County-provided dataset (polling centre lists, ward boundaries, local officer lists): `dataset_id`, `mou_id`, `county_code`, `dataset_type`, `received_at`, `validated_at`, `approved_by`, `file_hash` (SHA-256). | B |
| **MinistryApproval** | Formal approval from Ministry of Interior or other ministry: `approval_id`, `mou_id`, `ministry`, `ref_number`, `approved_at`, `document_url`. | B |
| **CoordinationIncident** | Cross-agency incident: `incident_id`, `county_code`, `severity` (LOW / MEDIUM / HIGH / CRITICAL), `category` (SECURITY / LOGISTICS / ELECTORAL / INTER-AGENCY), `description`, `reported_by`, `escalated_to_ecc_at`, `resolved_at`. | A |

## 3. External integrations

| Integration | Direction | Auth | SLA | Failure handling |
|---|---|---|---|---|
| **County government IdPs** (47 counties) | Inbound SAML 2.0 federation | SAML metadata exchange per MoU | Real-time SSO | Fall back to local JWT if county IdP unavailable; alert coordination officer |
| **ECC (M16)** | Outbound event (`eel.governance.critical_incident.v1`) | Kafka SASL/SCRAM | < 60 s for CRITICAL severity | Retry with exponential backoff; alert if ECC not acking within 5 min |
| **Ministry of Interior API** | Bidirectional (approval status + security coordination) | SVID mTLS | Per Ministry SLA | Retry; manual escalation via coordination officer |
| **GIS boundary service (M18)** | Inbound (county boundary validation) | Internal gRPC | Synchronous | Hard fail: dataset rejected if boundary validation fails |

## 4. Pipeline

Files on disk under `services/governance-svc/crates/coordination/src`: `lib.rs`, `models.rs`, `mou.rs`, `dataset.rs`, `incident.rs`, `routes/`.

```
coordination officer creates MoU (counterparty + scope)
      -> legal officer reviews; commissioner signs with HSM
      -> emit eel.governance.mou_signed.v1
      -> SAML federation metadata exchanged with county IdP
      -> county officer (authenticated via county SAML IdP) uploads CountyDataset
            -> file_hash computed (SHA-256); boundary validation via M18 gRPC
            -> coordination officer reviews and approves
            -> emit eel.governance.county_dataset_approved.v1
      -> Ministry of Interior issues MinistryApproval
            -> officer records approval reference number + document
            -> emit eel.governance.ministry_approval_received.v1
      -> coordination officer or field officer reports CoordinationIncident
            -> severity classification: LOW / MEDIUM / HIGH / CRITICAL
            -> CRITICAL: emit eel.governance.critical_incident.v1
               -> ECC (M16) receives event within 60 s SLA
            -> incident resolved: emit eel.governance.incident_resolved.v1
```

SAML federation is handled by the SAML middleware in `packages/eel-saml-rs`, which wraps the `samael` crate (Samael Project, open-source, Apache 2.0).

## 5. API surface

Contract: `packages/openapi-contracts/governance/coordination.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/coordination/mous` | Coordination officer (JWT + SpiceDB `COORDINATION_OFFICER`) | Create MoU |
| `PATCH /v1/coordination/mous/{id}/sign` | Commissioner + HSM | Sign MoU |
| `POST /v1/coordination/mous/{id}/datasets` | County officer (SAML JWT) | Upload county dataset |
| `PATCH /v1/coordination/mous/{id}/datasets/{did}/approve` | Coordination officer | Approve dataset |
| `POST /v1/coordination/mous/{id}/approvals` | Coordination officer | Record ministry approval |
| `POST /v1/coordination/incidents` | Any authenticated officer | Report incident |
| `PATCH /v1/coordination/incidents/{id}/escalate` | Coordination officer | Escalate to ECC |
| `PATCH /v1/coordination/incidents/{id}/resolve` | Incident commander | Resolve incident |
| `GET /v1/coordination/incidents` | ECC system (SVID) | Feed for M16 map |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Coordination console (Django) | `services/governance-svc/django/coordination_admin` | Manage MoUs, datasets, ministerial approvals |
| Incident reporting (LiveView) | `services/governance-svc/ex/incident_web` | Report, escalate, resolve incidents |
| County officer portal (Astro) | `services/governance-svc/astro/county-portal` | County IdP login, dataset upload |

Languages: EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **MoU document digest**: the commissioner HSM signature covers `SHA-256(mou_id || counterparty_name || scope_summary || document_url)`. Any alteration to the document URL (pointing to a different file) breaks the signature.
- **Dataset hash verification**: `CountyDataset.file_hash` is computed at upload and re-verified on every read; a nightly job re-hashes all stored files and alerts on mismatch.
- **SAML assertion replay protection**: `packages/eel-saml-rs` rejects assertions with a `NotOnOrAfter` in the past and maintains a 24-hour assertion ID cache to block replays (Cantor et al., OASIS SAML 2.0, 2005).
- **CRITICAL incident SLA**: Prometheus alert `eel_governance_critical_incident_ecc_ack_seconds > 300` fires if ECC does not acknowledge within 5 minutes.
- **Audit shadow**: every aggregate mutation logged to `eel_governance_audit_shadow`.

## 8. Kenya statute alignment

- **Elections Act §24 (county coordination)**: MoUs with county governments are a statutory requirement for access to county infrastructure on polling day.
- **Intergovernmental Relations Act 2012 §30 (MoU framework)**: MoU structure follows the IRA 2012 template for intergovernmental cooperation agreements.
- **National Cohesion and Integration Act 2008 (incident classification)**: CRITICAL incidents in category `ELECTORAL` are automatically flagged for National Cohesion and Integration Commission (NCIC) notification (see M23 Grievance Channels).
- **Kenya Data Protection Act §26 (minimisation)**: county datasets containing personal data (officer lists) are stored with column-level encryption; only the coordination officer role can decrypt.

## 9. Honest caveats + external prerequisites

- SAML federation with all 47 county IdPs requires each county government IT department to configure metadata exchange. SAML metadata for all 47 counties must be provisioned before election cycle begins; the `packages/eel-saml-rs` library is ready.
- Ministry of Interior API integration is via a bilateral MoU; the adapter is written to the documented Ministry API version. If the Ministry changes its API, the adapter requires updating.
- The CRITICAL incident SLA (60 s to ECC) assumes Kafka replication and M16 consumer are healthy. Degraded Kafka (e.g., leader election) can breach the SLA; the runbook covers manual escalation via satellite phone if Kafka is unavailable.
- Incident category taxonomy (`SECURITY / LOGISTICS / ELECTORAL / INTER-AGENCY`) is defined in `services/governance-svc/config/incident_taxonomy.toml`. Changes to this taxonomy require a code migration.

## 10. Test coverage + operational runbooks

Coverage under `services/governance-svc/crates/coordination/tests/`: MoU signing and HSM verification (integration, testcontainers), SAML assertion replay protection (property-based), incident escalation round-trip to ECC mock (pytest-httpserver), dataset hash verification (property-based), CRITICAL SLA regression (chaos via `chaos/scenarios/eel-coordination-ecc-*.yaml`).

Runbooks under `docs/runbooks/`:

- `governance-svc.md` (section: county-national-coordination) -- deploy, rollback.
- `county-idp-federation-failure.md` -- fallback to local JWT when county SAML IdP is unavailable.
- `critical-incident-ecc-sla-breach.md` -- manual ECC escalation procedure.

Related documents: full spec at `docs/plans/modules/county-national-coordination.md`; service capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`; M16 ECC at `docs/features/election-command-centre.md`.
