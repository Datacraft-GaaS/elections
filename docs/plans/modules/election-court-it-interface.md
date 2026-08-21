# M40 election-court-it-interface

| field | value |
|---|---|
| M-number | M40 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | cycle-critical 99.9%; 99.99% from T+1 to T+90 (petition window and appeals) |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M40-election-court-it-interface.md |

## Purpose

Provide the Supreme Court IT Registrar, High Court election judges, and registered petitioners with pre-emptive authenticated access to IEBC's evidentiary record. Assemble and publish the fifteen-item evidentiary bundle specified in Strategy §XII.4 within 24 hours of election-day close. Maintain a pre-registered technical-witness portal. This module directly addresses the Raila v IEBC [2017] eKLR para 272 "declined access" finding: access is architecturally guaranteed, not discretionary.

Strategy reference: §XII.4 (fifteen evidentiary bundle items), §XII (election petition management). Constitution Art 140; Elections Act 2011 §75-§85.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| IEBC Legal Bundle Publication Console | Phoenix LiveView | services/governance-svc/ex/court_web/ |
| Petitioner Portal (WCAG 2.2 AA) | Astro | services/governance-svc/astro/court-portal/ |
| Public Bundle Index (WCAG 2.2 AA) | Astro | services/governance-svc/astro/court-portal/ |
| Technical Witness Console (read-only) | Phoenix LiveView | services/governance-svc/ex/court_web/ |
| API | Rust (axum) | services/governance-svc/crates/governance-api/ |

Languages: EN + SW on all portal surfaces; bundle manifest in EN.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| BundleItem | id, bundle_id, item_number (1-15 per §XII.4), item_name, source_module, scope, file_id (encrypted blob ref), item_hash (SHA-256), signed_by, signed_at, scheduled_at, published_at, merkle_node_id | A | T1-audit |
| EvidenceBundle | id, election_id, items (BundleItem IDs), manifest_hash (SHA-256 of manifest JSON), manifest_merkle_node_id, legal_director_signed_at, published_at, publication_deadline (T+1 + 24 hours) | A | T1-audit |
| PetitionerAccess | id, petitioner_id, auth_method (JUDICIARY_ECOURT_OAUTH/IEBC_PORTAL_CERT), registered_at, petition_reference, items_requested, items_delivered | B | T1-consensus |
| SupplementaryRequest | id, requestor_id, requestor_type (COURT_REGISTRAR/PETITIONER/HIGH_COURT_JUDGE), item_type, scope, date_range_start, date_range_end, status (RECEIVED/APPROVED/ASSEMBLING/DELIVERED/DECLINED), legal_director_approved_at, delivered_at, decline_reason | B | T1-consensus |
| TechnicalWitness | id, officer_id, credential_type, testimony_areas, availability_start, availability_end, statement_template_ids, registered_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/governance/court.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/court/bundles | 2 | JWT+SpiceDB (IEBC_LEGAL) | yes | create evidence bundle record |
| POST /v1/court/bundles/{id}/items | internal gRPC | SOURCE_MODULE mTLS | yes | attach signed bundle item from source module |
| POST /v1/court/bundles/{id}/sign | 2 | JWT+SpiceDB (IEBC_LEGAL) + HSM | yes | Legal Director sign and publish bundle; gate: all 15 items attached |
| GET /v1/court/bundles/{id}/index | 1 | public | n/a | bundle index (item names, hashes, timestamps; no files) |
| GET /v1/court/bundles/{id}/items/{item_id} | 3 | JWT+SpiceDB (PETITIONER) | n/a | download signed bundle item file |
| POST /v1/court/petitioners | 2 | JWT+SpiceDB (IEBC_LEGAL) | yes | register petitioner with portal access |
| POST /v1/court/supplementary | 2 | JWT+SpiceDB (COURT_REGISTRAR) | yes | submit supplementary evidence request |
| PATCH /v1/court/supplementary/{id} | 2 | JWT+SpiceDB (IEBC_LEGAL) | yes | approve, assemble, or deliver supplementary item |
| POST /v1/court/witnesses | 2 | JWT+SpiceDB (IEBC_TECHNICAL) | yes | pre-register technical witness |
| GET /v1/court/witnesses/{id}/console | 2 | JWT+SpiceDB (TECHNICAL_WITNESS) | n/a | read-only evidence console; hearing support; access auto-logged |
| GET /v1/court/access-log | 2 | JWT+SpiceDB (IEBC_LEGAL) | n/a | petitioner download audit log |

All responses TLS 1.3; each response signed with IEBC public key.

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.bundle_item_attached.v1 | emits | eel.governance.court.v1 | audit | yes | at-least-once idempotent |
| eel.governance.bundle_published.v1 | emits | eel.governance.court.v1 | audit | yes | exactly-once |
| eel.governance.supplementary_delivered.v1 | emits | eel.governance.court.v1 | audit | yes | at-least-once idempotent |
| eel.governance.witness_console_accessed.v1 | emits | eel.governance.court.v1 | audit | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M35 CVR | internal | integrations/cvr/ | yes | item 1: signed voter register snapshot; retry with exponential back-off |
| M06 BVI Back-End | internal | integrations/bvi/ | yes | items 2, 8: biometric logs, turnout time-series |
| M10 RTS Back-End | internal | integrations/rts/ | yes | items 3, 4: Form 34 images and Merkle proofs |
| M39 Ballot Printing Chain | internal | integrations/ballot-chain/ | yes | item 5: chain-of-custody bundle per constituency |
| M08 RTS Device | internal | integrations/rts-device/ | yes | item 6: KIEMS kit audit logs |
| M26 Anomaly Detection | internal | integrations/anomaly/ | yes | item 7: sanitised anomaly event log |
| M38 Poll Worker Training LMS | internal | integrations/training/ | yes | item 10: worker certification log |
| M23 Grievance Channels | internal | integrations/grievance/ | yes | item 11: sanitised grievance log |
| M22 Campaign Finance | internal | integrations/campaign-finance/ | yes | item 12: compliance status at election day |
| M18 Boundary Transparency | internal | integrations/boundary/ | yes | item 13: delimitation dataset |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | bundle manifest anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | Legal Director signing; items signed within 60 s each |
| Kenya Judiciary eCourt OAuth | external | integrations/judiciary-oauth/ | yes (stub) | petitioner authentication; fallback: IEBC-issued portal certificate |
| M11 Public Portal | internal | integrations/portal/ | yes | public bundle index |

## ML and AI

- None.

## Dependencies

- Modules: M06, M08, M10, M11, M18, M22, M23, M26, M35, M38, M39
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

1. FR-M40-001: within 24 hours of election-day results-transmission close (T+1), system publishes a signed evidentiary bundle containing all fifteen §XII.4 items; publication is automatic, not petition-triggered
2. FR-M40-002: the fifteen bundle items are: (1) signed voter register snapshot, (2) biometric verification logs, (3) Form 34A/B/C images with cryptographic provenance, (4) Merkle roots and inclusion proofs, (5) ballot chain-of-custody bundle, (6) KIEMS kit audit logs, (7) sanitised anomaly event log, (8) voter turnout time-series, (9) network connectivity log, (10) poll worker certification log, (11) sanitised grievance log, (12) campaign finance compliance status, (13) boundary delimitation dataset, (14) technical witness register + credential hashes, (15) system audit trail of all IEBC officer actions on result records T-7 to T+1
3. FR-M40-003: each item individually HSM-signed; bundle manifest (JSON) signed over SHA-256 of each item; manifest hash anchored in P05 Merkle chain
4. FR-M40-004: registered petitioners (Kenya Judiciary eCourt OAuth or IEBC-issued certificate) can browse bundle index, request items, download signed packages, track delivery status
5. FR-M40-005: any specific log file requested by the Supreme Court IT Registrar or a registered petitioner is produced and delivered within 24 hours of the authenticated request
6. FR-M40-006: supplementary request workflow; Court Registrar submits structured request; IEBC Legal Director reviews and approves; system assembles and signs; delivery tracked
7. FR-M40-007: IEBC technical witnesses pre-register via portal (name, credential type, availability, testimony areas); system assigns credentials and generates signed statement template per area
8. FR-M40-008: during petition hearing, technical witness accesses read-only evidence console; console logs access events for court records
9. FR-M40-009: public bundle index (item names, timestamps, hashes) on M11 without login; item downloads require petitioner authentication; IAB observers can verify hashes independently
10. FR-M40-010: declined-access prevention: every bundle item available to registered petitioner by right; items not yet produced show scheduled-delivery timestamp; no item withheld without IEBC Legal Director override logged with reason
11. FR-M40-011: WCAG 2.2 AA on petitioner portal and public bundle index
12. FR-M40-012: EN + SW on all portal surfaces

## Acceptance criteria

1. AC-M40-01: all fifteen items published within 24 hours of T+1 close; bundle manifest signed and Merkle-anchored; integration test
2. AC-M40-02: petitioner downloads item; signature verified against IEBC public key; e2e test
3. AC-M40-03: supplementary request produced and delivered within 24 hours; load test
4. AC-M40-04: petitioner portal available at 99.99% from T+1 to T+90; uptime monitor
5. AC-M40-05: public bundle index accessible without login; item downloads blocked without petitioner JWT; security test
6. AC-M40-06: WCAG 2.2 AA on public bundle index and petitioner portal; screen reader + axe-core

## Applicable edge cases

**connectivity**
- EC-CON-01: P05 Merkle chain temporarily unavailable; bundle publication proceeds; Merkle anchor retried with exponential back-off; alert raised; eel-outbox-rs ensures anchor eventually delivered

**security**
- EC-SEC-01: unauthenticated user requests item file; blocked with 401; logged
- EC-SEC-08: petitioner attempts to access item outside their registered scope; SpiceDB denies; access attempt logged

**operations**
- EC-OPS-05: one source module fails to deliver item within 24 hours; per-item scheduled-delivery timestamp published; automated retry with alert; Legal Director may publish partial bundle with annotation
- EC-OPS-06: Legal Director attempts to sign manifest with fewer than fifteen items attached; gate rejects signing; UI shows remaining items

**legal**
- EC-LEG-04: Judiciary eCourt OAuth not ready; IEBC-issued portal certificate activates as fallback; no petitioner blocked

**human**
- EC-HUM-06: WCAG 2.2 AA on all public and petitioner-facing screens

**scale**
- EC-SCL-02: 1,000 concurrent authenticated petitioner downloads during 28-day petition window; CDN serves static item files; governance-api handles auth only

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | bundle-item hash computation, manifest signing logic, declined-access gate, 24-hour deadline calculator, sign gate (15-item completeness) | nextest | M40-UNIT- |
| contract | each source module (M06, M10, M26, M38, M39 etc.) pushes item; manifest sign triggers Merkle anchor | Pact | M40-PACT- |
| integration | T+1 event; automatic assembly; 15-item count; Legal Director sign; Merkle anchor | integration_test | M40-INT- |
| e2e | simulate T+1; verify bundle; register petitioner; download item; verify signature | nextest e2e | M40-E2E- |
| security | unauthenticated item download (deny); petitioner outside scope (deny); sign with < 15 items (deny) | ZAP + red team | M40-SEC- |
| a11y | WCAG 2.2 AA on public bundle index and petitioner portal | axe-core + screen reader | M40-A11Y- |
| load | 1,000 concurrent petitioner downloads; 28-day petition window | k6 | M40-LOAD- |
| chaos | P05 unavailable; bundle publishes; anchor retried; no data loss | netem | M40-CHAOS- |

Tags: `#[trace("FR-M40-010","AC-M40-01","EC-OPS-05")]`

## Runbooks and dashboards

- `docs/runbooks/governance-svc-degradation.md` - evidence bundle assembly failure; source-module retry paths; Legal Director override procedure
- `docs/runbooks/bundle-publication-incident.md` - new runbook; item delivery SLA breach; Commissioner escalation; partial-bundle publication procedure
- Grafana dashboard: `eel-iems-40` (alert ruleset: `iems-40-alerts`)
- SLO alerts: all 15 items published within 24 hrs of T+1; petitioner portal p99 <= 1 s; Merkle anchor within 60 s of sign event

## Open questions

- Kenya Judiciary eCourt OAuth MoU: external; IEBC-facilitated; if not signed by T-30, IEBC-issued portal certificate activates as permanent fallback; confirm fallback activation criteria with IEBC Legal.
- Items 9 and 15 (network connectivity log, officer audit trail T-7 to T+1): no named source module in the M-series; these must be generated by governance-svc itself from infrastructure logs and the M10 audit trail respectively; confirm data ownership and extract pipeline before T-180.
- Technical witness second-officer coverage: minimum two witnesses per testimony area required; pre-registration gate must enforce this; confirm with IEBC CTO.
