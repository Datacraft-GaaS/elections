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
| PetitionerAccess | id, petitioner_id, auth_method (JUDICIARY_ECOURT_OAUTH/COMMISSION_PORTAL_CERT), registered_at, petition_reference, items_requested, items_delivered | B | T1-consensus |
| SupplementaryRequest | id, requestor_id, requestor_type (COURT_REGISTRAR/PETITIONER/HIGH_COURT_JUDGE), item_type, scope, date_range_start, date_range_end, status (RECEIVED/APPROVED/ASSEMBLING/DELIVERED/DECLINED), legal_director_approved_at, delivered_at, decline_reason | B | T1-consensus |
| TechnicalWitness | id, officer_id, credential_type, testimony_areas, availability_start, availability_end, statement_template_ids, registered_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/governance/court.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/court/bundles | 2 | JWT+SpiceDB (COMMISSION_LEGAL) | yes | create evidence bundle record |
| POST /v1/court/bundles/{id}/items | internal gRPC | SOURCE_MODULE mTLS | yes | attach signed bundle item from source module |
| POST /v1/court/bundles/{id}/sign | 2 | JWT+SpiceDB (COMMISSION_LEGAL) + HSM | yes | Legal Director sign and publish bundle; gate: all 15 items attached |
| GET /v1/court/bundles/{id}/index | 1 | public | n/a | bundle index (item names, hashes, timestamps; no files) |
| GET /v1/court/bundles/{id}/items/{item_id} | 3 | JWT+SpiceDB (PETITIONER) | n/a | download signed bundle item file |
| POST /v1/court/petitioners | 2 | JWT+SpiceDB (COMMISSION_LEGAL) | yes | register petitioner with portal access |
| POST /v1/court/supplementary | 2 | JWT+SpiceDB (COURT_REGISTRAR) | yes | submit supplementary evidence request |
| PATCH /v1/court/supplementary/{id} | 2 | JWT+SpiceDB (COMMISSION_LEGAL) | yes | approve, assemble, or deliver supplementary item |
| POST /v1/court/witnesses | 2 | JWT+SpiceDB (COMMISSION_TECHNICAL) | yes | pre-register technical witness |
| GET /v1/court/witnesses/{id}/console | 2 | JWT+SpiceDB (TECHNICAL_WITNESS) | n/a | read-only evidence console; hearing support; access auto-logged |
| GET /v1/court/access-log | 2 | JWT+SpiceDB (COMMISSION_LEGAL) | n/a | petitioner download audit log |

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

Bundle publication pipeline:
1. FR-M40-001: within 24 hours of election-day results-transmission close (T+1), system publishes signed evidentiary bundle containing all fifteen §XII.4 items; publication automatic, not petition-triggered
2. FR-M40-013: bundle assembly pipeline triggers automatically at T+1 results-transmission-close event; source modules submit items via gRPC mTLS; per-item status tracked in EvidenceBundle
3. FR-M40-014: per-item scheduled-delivery timestamp published in bundle index; updated when item arrives; petitioner can subscribe to delivery notification for specific items
4. FR-M40-015: Legal Director sign gate: signing manifest requires all 15 items attached; UI shows remaining items count; partial bundle cannot be signed except with explicit Legal Director override
5. FR-M40-016: Legal Director override (partial publication): requires Commissioner co-signature; override reason logged to immutable audit trail; partial bundle annotated as provisional
6. FR-M40-017: bundle manifest Merkle anchor completes within 60 s of Legal Director sign event; if P05 unavailable, eel-outbox-rs queues anchor request; bundle publication proceeds; anchor confirmed later

The fifteen bundle items (§XII.4):
7. FR-M40-002: item 1: signed voter register snapshot from M35; item 2: biometric verification logs per station from M06; item 3: Form 34A/B/C images with cryptographic provenance from M10; item 4: Merkle roots and inclusion proofs for all result forms
8. FR-M40-018: item 5: ballot chain-of-custody bundle per constituency from M39; item 6: KIEMS kit audit logs per station from M08
9. FR-M40-019: item 7: sanitised anomaly event log from M26; item 8: voter turnout time-series per station from M06
10. FR-M40-020: item 9: network connectivity log (uptime per station, aggregated) generated by governance-svc from infrastructure logs
11. FR-M40-021: item 10: poll worker certification log from M38; all workers who served; station code included; worker ID masked to last 4 digits
12. FR-M40-022: item 11: sanitised grievance and election offences case log from M23; PII sanitised before attachment; M23 confirms sanitisation
13. FR-M40-023: item 12: campaign finance compliance status at election day from M22 per candidate/party; item 13: boundary delimitation dataset from M18 at election date
14. FR-M40-024: item 14: technical witness register + credential hashes generated by M40 from TechnicalWitness records; hashes only, not certificates
15. FR-M40-025: item 15: system audit trail of all IEBC officer actions on result records from T-7 to T+1; generated by governance-svc from M10 audit trail; signed by Legal Director

Bundle integrity:
16. FR-M40-003: each item individually HSM-signed via P06; bundle manifest (JSON) signed over SHA-256 of each item; manifest hash anchored in P05 Merkle chain; signing completes within 60 s per item
17. FR-M40-026: each item hash published to public bundle index at time of item signing; IAB observers can verify hashes independently before bundle is complete

Petitioner portal:
18. FR-M40-004: registered petitioners (Kenya Judiciary eCourt OAuth or IEBC-issued certificate) can browse bundle index, request items, download signed packages, track delivery status
19. FR-M40-031: petitioner registration requires Kenya Judiciary eCourt OAuth token or IEBC-issued portal certificate; registration reviewed by IEBC Legal; portal access granted within 4 hours of petition filing
20. FR-M40-032: each petitioner registration linked to petition reference; petitioner can only download items relevant to their petition scope; SpiceDB enforces scope
21. FR-M40-033: petitioner download generates signed download receipt; receipt includes item ID, hash, timestamp, petitioner ID (hashed); receipt stored immutably for court records
22. FR-M40-034: petitioner portal tracks delivery status per item requested; status states: SCHEDULED, AVAILABLE, DOWNLOADED; timestamp for each transition
23. FR-M40-035: bulk download option: petitioner can request all 15 items as a single signed ZIP; ZIP manifest signed by IEBC Legal Director; download tracked

Supplementary requests:
24. FR-M40-005: any specific log file requested by Supreme Court IT Registrar or registered petitioner produced and delivered within 24 hours of authenticated request
25. FR-M40-006: supplementary request workflow; Court Registrar submits structured request; IEBC Legal Director reviews and approves; system assembles and signs; delivery tracked
26. FR-M40-036: supplementary request form: item type picker (from standard taxonomy), scope (national/county/constituency/station), date range, justification text; authenticated submission only
27. FR-M40-037: standard supplementary items (known types) auto-assembled by pipeline; Legal Director approves at batch level; non-standard items require individual review
28. FR-M40-038: supplementary item signed by IEBC Legal Director via P06 HSM; signing logged; item hash published to bundle index
29. FR-M40-039: supplementary request status: RECEIVED, APPROVED, ASSEMBLING, DELIVERED, DECLINED; status visible to requesting party in real time
30. FR-M40-040: declined supplementary request requires Legal Director to record decline reason in immutable audit trail; decline reason visible to petitioner

Technical witnesses:
31. FR-M40-007: IEBC technical witnesses pre-register via portal; system assigns credentials and generates signed statement template per testimony area
32. FR-M40-041: minimum two technical witnesses pre-registered per testimony area; pre-registration gate enforced; alert to IEBC CTO if any area falls below minimum
33. FR-M40-042: signed witness statement template generated per testimony area; template hash stored; witness can view and download template at any time
34. FR-M40-043: evidence console for witnesses: read-only access to all items relevant to registered testimony areas; search by station code; access events auto-logged with timestamp and IP
35. FR-M40-044: witness portal credentials expire at T+90; credentials not renewable beyond petition window without Legal Director authorisation
36. FR-M40-008: during petition hearing, technical witness accesses read-only evidence console; console logs access events for court records with actor, item, timestamp, IP

Declined-access prevention:
37. FR-M40-010: declined-access prevention: every bundle item available to registered petitioner by right; items not yet produced show scheduled-delivery timestamp; no item withheld without Legal Director override logged with reason
38. FR-M40-009: public bundle index (item names, timestamps, hashes) on M11 without login; item downloads require petitioner authentication; IAB observers can verify hashes independently

Access log and audit:
39. FR-M40-045: petitioner download audit log includes: petitioner ID (hashed), item ID, item hash, download timestamp, IP address; retained 7 years; exportable by IEBC Legal
40. FR-M40-046: every technical witness access event logged: witness ID, item viewed, timestamp, IP; log immutable; available to court on request
41. FR-M40-047: IEBC Legal Director supplement approval logged: approver ID, request ID, decision, timestamp; all logged in immutable audit trail anchored in P05

Availability and performance:
42. FR-M40-048: petitioner portal CDN-backed for static item files; governance-api handles authentication only; p99 <= 1 s under 1,000 concurrent authenticated users
43. FR-M40-049: petitioner portal availability >= 99.99% from T+1 to T+90; SLO alert at 99.95% threshold; auto-scale on pod CPU > 70%
44. FR-M40-050: bundle index public endpoint: cached with 5-minute TTL; cache invalidated on each new item publication; p99 <= 500 ms; CDN edge hit ratio >= 90%

Accessibility and language:
45. FR-M40-011: WCAG 2.2 AA on petitioner portal and public bundle index; independent accessibility audit at OAT gate
46. FR-M40-012: EN + SW on all portal surfaces; bundle manifest in EN; language selector on portal landing page

Security:
47. FR-M40-027: all bundle items served over TLS 1.3; bundle items individually signed and independently verifiable against IEBC public key; no unsigned item served

Observability:
48. FR-M40-028: OpenTelemetry traces exported to central collector; Grafana dashboard eel-iems-40; alert ruleset iems-40-alerts
49. FR-M40-029: SLO alerts: all 15 items published within 24 hrs of T+1; petitioner portal p99 <= 1 s; Merkle anchor within 60 s of sign event; portal availability >= 99.99%

Retention:
50. FR-M40-030: all bundle items, access logs, supplementary requests retained 7 years per Elections Act; encrypted at rest; transferred to M69 Long-Term Archive at decommission

## Acceptance criteria

1. AC-M40-01: all fifteen items published within 24 hours of T+1 close; bundle manifest signed and Merkle-anchored; integration test confirms all three conditions
2. AC-M40-02: petitioner downloads item; signature verified against IEBC public key; e2e test confirms independent verification
3. AC-M40-03: supplementary request produced and delivered within 24 hours; load test confirms at 200 concurrent requests during petition window simulation
4. AC-M40-04: petitioner portal available >= 99.99% from T+1 to T+90; uptime monitor; SLO alert at 99.95% threshold
5. AC-M40-05: public bundle index accessible without login; item downloads blocked without petitioner JWT; security test confirms no bypass
6. AC-M40-06: WCAG 2.2 AA on public bundle index and petitioner portal; screen reader + axe-core + manual audit at OAT gate
7. AC-M40-07: Legal Director cannot sign manifest with fewer than 15 items attached; gate test confirms 422 returned with missing item count
8. AC-M40-08: petitioner portal p99 <= 1 s under 1,000 concurrent authenticated users; k6 load test during 28-day petition window simulation
9. AC-M40-09: every bundle item and manifest anchored in P05 Merkle chain within 60 s of signing; 100% anchoring confirmed by Merkle reconciliation
10. AC-M40-10: minimum two technical witnesses pre-registered per testimony area by T-7; witness registry confirms; alert fires if minimum not met at T-14
11. AC-M40-11: zero unauthorised item access in penetration test; access-log audit confirms no unauthenticated downloads; ZAP scan confirms
12. AC-M40-12: petitioner download audit log retained 7 years; all required fields present; no PII in unscoped logs; retention policy audited at OAT gate

## Applicable edge cases

data:
- EC-DAT-01: duplicate bundle item attachment with same idempotency key; cached response returned; Merkle chain not re-anchored for duplicate
- EC-DAT-02: same bundle item key with different body hash (tamper attempt); 422 idempotency-key-reuse returned; tamper logged to audit trail
- EC-DAT-09: evidence item file names with Unicode characters (constituency names in Swahili); NFC normalization applied; no file system truncation in bundle index table

security:
- EC-SEC-01: bundle manifest byte-flipped; Merkle proof verification fails; tamper alert raised to IEBC Legal within 30 s; item locked in audit log
- EC-SEC-03: P06 HSM unavailable during bundle signing; signing queued; bundle publication blocked until HSM recovers; alert raised; Commissioner notified of delay
- EC-SEC-05: injection attempt in supplementary request field; parameterised queries only; ZAP scan; Semgrep CI at OAT gate
- EC-SEC-08: XXE injection attempt in supplementary request XML; DTD disabled; strict JSON parsing; fuzz corpus in CI
- EC-SEC-16: petitioner IDs and officer IDs stripped from unscoped telemetry; Tier A/B fields absent; log-scanner fixture confirms; IEBC CISO audit

identity:
- EC-IDN-02: petitioner JWT expires mid-download session; in-flight file transfer completes; next request 401 + silent refresh; no file truncated
- EC-IDN-09: IEBC Legal Director approver revoked mid-signing session; re-validation required; partial signing halted; no bundle published without valid Legal Director credential
- EC-IDN-10: SpiceDB unavailable; all petitioner item access attempts fail closed 503; public bundle index continues from CDN cache; alert raised to SRE

scale:
- EC-SCL-01: 1,000 concurrent authenticated petitioner downloads during 28-day petition window; CDN serves static item files; governance-api handles auth only; p99 <= 1 s maintained
- EC-SCL-02: 100k independent verifiers checking bundle index hashes simultaneously; ETag 304 responses applied; rate-limit enforced; SDK jitter reduces thundering herd
- EC-SCL-04: Kafka consumer lag > 60 s on eel.governance.court.v1 topic; alert raised; backpressure applied; eel-outbox-rs retains events; no bundle item loss

connectivity:
- EC-CON-01: P05 Merkle chain temporarily unavailable; bundle publication proceeds; eel-outbox-rs queues anchor request; alert raised; anchor eventually delivered; no bundle-item loss

operations:
- EC-OPS-05: one source module fails to deliver item within 24 hours; per-item scheduled-delivery timestamp published; automated retry with alert; Legal Director may publish partial bundle with annotation
- EC-OPS-10: PITR restore during petition window; restore operation logged to audit trail; RPO <= 5 min; bundle re-anchored after restore; petitioner access restored within RTO

security (additional):
- EC-SEC-04: KEK rotation for encrypted bundle item blobs mid-petition window; online re-wrap; old key readable until rotation complete; no petitioner download gap during rotation
- EC-SEC-09: rate-limit evasion on bundle index public endpoint; keyed by IP/64 subnet; 500/min enforced; excess requests shed with 429; Nuclei test confirms
human:
- EC-HUM-06: WCAG 2.2 AA on petitioner portal and public bundle index; screen reader + keyboard navigation complete; axe-core + manual audit at OAT gate
- EC-HUM-07: KSL video captions included in bundle item 9 (network log explanatory document) when video format used; caption asset presence checked before item published
- EC-HUM-08: EN and SW portal surfaces; long Swahili strings in item names and supplementary request forms; no truncation in bundle index table; ICU plurals applied

legal:
- EC-LEG-04: Judiciary eCourt OAuth not operational at T+1; IEBC-issued portal certificate activates as fallback; no petitioner blocked; fallback activation logged and Commissioner notified
- EC-LEG-08: supplementary evidence request arrives at T+6d 23h; 24-hour production clock applies; not freeze-blocked; item delivered within statutory window; timed integration test confirms
- EC-TIM-08: petition filing deadline at T+28 midnight EAT; boundary enforced inclusive; petitioner portal remains accessible through midnight; late access attempt logged and served if within window

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
