# Campaign finance transparency (M22)

Module M22 (`campaign-finance-transparency`) owned by `transparency-svc`. Accepts statutory campaign finance disclosures from candidates and parties, cross-references them against CBK/FRC aggregated cash data, flags ceiling breaches and anomaly patterns, publishes aggregated public reports anchored in the Merkle log, and assembles HSM-signed evidence packages routed to ODPP and DCI under IEBC Chair authority.

## Contents

1. What campaign finance transparency does (vs related modules)
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

## 1. What campaign finance transparency does (vs related modules)

- **M12 CRMS** = candidate registration and accreditation; M22 consumes candidate identity from CRMS to pre-populate ceiling tiers.
- **M22** = post-registration statutory finance disclosure: ingest weekly spending returns, compute ceilings, cross-reference with CBK/FRC constituency cash data, flag anomalies, route evidence to regulators.
- **M23 Grievance channels** = public complaint intake; finance complaints from M23 feed M22 as additional anomaly signals.

M22 is the only module that can initiate an ODPP/DCI referral for election finance offences. All anomaly adjudication requires a named human officer; no automated referral occurs without officer review and IEBC Chair HSM signature.

## 2. Data plane

Five aggregates (per `services/transparency-svc/crates/finance-api/src/models.rs` and the M22 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **CandidateDisclosure** | Per-period submission: contributions_kes, expenditures_kes, bank-statement references, OCR-extracted totals, submission signature. | C |
| **CashAggregate** | Cumulative declared spending vs ECFA ceiling: total_in, total_out, ceiling_kes, status (BELOW/AMBER/BREACH), computed_at. | D |
| **AnomalyFlag** | Detected anomaly linked to a disclosure or CBK cross-reference: reason, severity, cbk_ref, status (OPEN/ADJUDICATED/DISMISSED). | C |
| **EvidencePackage** | HSM-signed PDF + JSON for regulator handoff: content_refs, chain_of_custody, sig, assembled_at. | C |
| **AuditEvent** | Every IEBC officer action: actor, action, ts_hlc, sig (append-only). | C |

Migrations land under `services/transparency-svc/migrations/`.

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Candidate bank statement | Weekly | PDF or OFX | JWT (candidate portal) | OCR retry 3x; officer override if extraction fails |
| M-Pesa Paybill CSV | Weekly | CSV | JWT (candidate portal) | Parser rejects malformed; candidate notified |
| Itemised expense receipts | Weekly | PDF | JWT (candidate portal) | OCR extracts totals; candidate confirms before submission |
| CBK/FRC aggregated cash-out | Daily | REST/JSON under MoU | SVID mTLS | Circuit breaker; heat map shows last-known data with staleness warning; alert to IEBC finance desk if lag >48 h |
| M23 grievance finance signals | Real-time | Kafka event | Internal | At-least-once idempotent; queue on broker failure |

Party finance officers may file on behalf of multiple candidates under one login; a separate `CandidateDisclosure` record with a per-candidate idempotency key is created for each.

## 4. Pipeline

Files on disk under `services/transparency-svc/crates/finance-api/src` and `crates/finance-anomaly/src`:

```
candidate portal upload (PDF/OFX/CSV)
  -> OCR extraction (crates/finance-anomaly/src/ocr.rs)
  -> candidate confirmation screen (diff extracted vs declared totals)
  -> CandidateDisclosure persisted; eel.finance.disclosure_submitted.v1 emitted
  -> ceiling check (ECFA 2013 s.22 tier from CRMS M12)
        BELOW  -> no action
        AMBER  -> flag at 80%; portal warning displayed
        BREACH -> AnomalyFlag(severity=RED) created;
                  ODPP referral queue entry within 60 s;
                  eel.finance.anomaly_flagged.v1 emitted
  -> daily CBK/FRC cross-reference (crates/finance-anomaly/src/cbk.rs)
        z-score > 2 SD from 30-day baseline
          -> FRC SAR alert draft within 30 min
        POCAMLA 2009 patterns (bulk cash-out near stations, rapid round-trip,
                               structuring)
          -> AnomalyFlag; IEBC finance desk notified
  -> officer adjudication queue (django/finance-admin/)
  -> IEBC Chair HSM signature required before EvidencePackage routed to ODPP/DCI
  -> eel.finance.evidence_assembled.v1 (exactly-once)
  -> P05 Merkle anchor within 60 s of package creation
```

Every write goes through the transactional outbox (`packages/eel-outbox-rs`). Debezium publishes to `eel.finance.disclosure.v1` and `eel.finance.anomaly.v1` topics.

## 5. API surface

Contract: `packages/openapi-contracts/transparency/finance.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /finance/v1/disclosures` | Candidate / party officer (JWT) | Submit disclosure for a period |
| `GET /finance/v1/disclosures/{id}` | Candidate (JWT) | Disclosure status |
| `GET /finance/v1/candidates/{id}/summary` | Public (CDN-cached) | Aggregate ceiling status and spending percentage (no raw transactions) |
| `GET /finance/v1/reports` | Public (CDN-cached) | Published election-cycle finance report |
| `GET /finance/v1/flags` | IEBC finance desk (JWT + SpiceDB) | Anomaly flag feed |
| `PATCH /finance/v1/flags/{id}` | IEBC finance desk | Adjudicate anomaly flag (accept / dismiss / escalate) |
| `POST /finance/v1/evidence/{candidate_id}` | IEBC officer (JWT + SpiceDB) | Assemble evidence package |
| `GET /finance/v1/evidence/{id}` | IEBC officer / FRC analyst | Evidence package for regulator |

Every write is response-signed via the middleware in `packages/eel-api-server-rs`. Every access is audited into `eel_finance_audit_shadow`.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Candidate disclosure portal (Astro) | `services/transparency-svc/astro/finance-portal/` | Weekly upload wizard; ceiling status; compliance deadline calendar |
| Anomaly adjudication back-office (Django) | `services/transparency-svc/django/finance-admin/` | IEBC finance desk reviews flags; assembles evidence packages |
| Public spending dashboard (Astro) | `services/transparency-svc/astro/finance-portal/` (public route) | Bar chart spending vs ceiling per candidate; constituency choropleth cash heat-map (z-score coloured); no raw bank data |
| Read API (Rust axum) | `services/transparency-svc/crates/finance-api/` | REST surface for internal and external consumers |

Languages: EN + SW for candidate portal and public dashboard; WCAG 2.2 AA across all human surfaces. Raw bank statements encrypted at rest (AES-256) and never exposed to any portal.

## 7. Integrity + anti-tamper

- **Merkle-anchored evidence packages** -- every `EvidencePackage` is HSM-signed by the IEBC Chair (per-cycle KEK from `eel_kek_generations`) and its hash becomes a leaf in the P05 transparency log. Courts can verify the inclusion proof without system access.
- **Dual-officer control** -- IEBC Chair HSM signature is required before an evidence package is routed to ODPP or DCI; a single officer attempt returns HTTP 403 and an audit log entry.
- **SAR pipeline** -- CBK/FRC spike alerts produce a draft SAR for FRC analyst review before submission; no automated SAR submission occurs without analyst sign-off.
- **Ceiling computation audit** -- the valuation method for in-kind contributions is stored as a versioned configuration record; every ceiling computation is logged with the configuration version used so historical recomputation is possible.
- **Idempotency** -- duplicate disclosure submission (same idempotency key + body) returns the cached response; the ceiling total is not double-counted. Same key with a different body returns HTTP 422 `idempotency-key-reuse`.
- **PII minimisation** -- raw bank transactions are never stored beyond the cross-reference window; bank statements are encrypted at rest and accessible only to the filing candidate and the IEBC Compliance Officer under logged, purpose-coded access.
- **Circuit breakers** on CBK/FRC and CRMS adapters via `packages/eel-resilience-rs` prevent upstream outages from cascading into the disclosure pipeline.

## 8. Kenya statute alignment

- **ECFA 2013 s.22** -- spending ceiling per contest tier; M22 enforces amber flag at 80% and red breach flag at 100%; ceiling tier sourced from CRMS M12.
- **ECFA 2013 ss.32-34** -- foreign money prohibition; inbound cross-border transfers flagged on candidate bank statement; auto-referral to CBK/FRC.
- **Elections Act s.98** -- general finance offence; ceiling computation and referral basis logged against this provision.
- **POCAMLA 2009** -- anti-money-laundering pattern detection applied to constituency cash-out data; SAR drafts reference POCAMLA suspicious-activity criteria.
- **Whistleblower Protection Act 2022** -- whistleblower tips routed through M23 protected channel; tip identity stored hashed; legal-privilege annotation on records; tip records routed to DCI/ODPP within 6 h via evidence package endpoint.
- **DPA 2019 s.26 (minimisation)** -- raw banking data not retained beyond cross-reference window; bank statements encrypted; individual transaction data not visible to any party other than the filing candidate and IEBC Compliance Officer.

## 9. Honest caveats + external prerequisites

- CBK/FRC MoU has not been signed as of spec writing. The data-sharing agreement, data minimisation requirements, and liability framework must be confirmed before the CBK cross-reference is enabled in production. The adapter and stub are ready under `integrations/cbk/`.
- The ceiling computation method for in-kind contributions is not fully specified in Elections Act s.98; IEBC legal must confirm the valuation method before the ceiling check can go live.
- OCR extraction accuracy on non-standard bank statement layouts is untested on all Kenyan commercial bank formats; the candidate-confirmation step is the mandatory human checkpoint before any ceiling computation uses extracted values.
- The ECFA prosecution registry (public-facing list of s.12 cases) depends on IEBC Compliance Officer manually recording each judgment outcome within 14 days; no automated court-feed integration is in scope.
- WhatsApp whistleblower tip intake is shared with M23 and depends on the M23 WhatsApp Cloud API approval (6-8 week lead time).

## 10. Test coverage + operational runbooks

Every disclosure path has coverage under `services/transparency-svc/crates/finance-api/tests/` and `crates/finance-anomaly/tests/`: unit tests for the ceiling check, anomaly rules, CBK cross-reference parser, evidence package builder, and purge scheduler; property-based tests (proptest) asserting ceiling breach always flags; contract tests (Pact) for M12 candidate identity and M23 grievance signal; integration tests with CBK stub, M12 double, SoftHSM evidence signing, and tlog anchor; e2e Playwright test covering the full flow from disclosure submit through ceiling check, CBK cross-reference, anomaly flag, adjudication, and evidence package assembly; load tests (k6) for the campaign finance period close with concurrent disclosures.

Runbooks under `docs/runbooks/`:

- `finance-transparency-degradation.md` (create): CBK unavailable; cross-reference lag; HSM fail; FRC handoff failure.
- `pgbackrest-pitr.md` (extend): PITR drill for finance data (T2 store, RPO 15 min, RTO 60 min).

Dashboards: `platform/observability/transparency-svc/grafana-finance.json` (disclosure rate, ceiling breach count, CBK cross-ref lag, anomaly flag queue depth). SLO alerts: `platform/observability/transparency-svc/slo-alerts.yaml` (99.9% burn rate; CBK lag >48 h; ceiling breach unacknowledged >4 h).

Related documents: full spec at `docs/plans/modules/campaign-finance-transparency.md`; service capability catalogue at `services/transparency-svc/docs/CAPABILITIES.md`; edge-case matrix rows tagged EC-FIN-* in `docs/plans/08-edge-case-matrix.md`.
