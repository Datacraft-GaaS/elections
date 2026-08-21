# M56 county-national-coordination

| field | value |
|---|---|
| M-number | M56 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M56-county-national-coordination.md |

## Purpose

Manages MoUs, county-level dataset approvals, ministerial coordination, and cross-tier incident escalation between county IEBC offices and the national Commission, with SAML brokering for heterogeneous county government identity providers.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| County coordination portal | Astro | astro/governance-public/ (county subdomain) |
| Admin coordination console | Django | django/governance-admin/ |
| Internal API | Rust (axum) | crates/county-coord/ |
| SAML broker | Elixir (Phoenix) | ex/saml_broker/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| MoU | id, county_code, ministry_code, effective_date, expiry_date, content_url, status (draft/signed/active/expired), signed_at | C | T1-consensus |
| CountyDataset | id, county_code, dataset_type, source_system, approved_by, approval_date, dataset_url | C | T1-consensus |
| MinistryApproval | id, mou_id, ministry_ref, approved_by, approved_at | C | T1-audit |
| CoordinationIncident | id, county_code, incident_type, severity, reported_at, resolved_at, ecc_alert_ref | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/coordination/mous | 3 accredited | JWT+SpiceDB | n/a | list MoUs |
| POST /v1/coordination/mous | 3 accredited | JWT+SpiceDB | yes | create MoU |
| PUT /v1/coordination/mous/{id}/sign | 3 accredited | JWT+SpiceDB | yes | dual-sign MoU |
| GET /v1/coordination/datasets | 3 accredited | JWT+SpiceDB | n/a | list county datasets |
| POST /v1/coordination/datasets | 3 accredited | JWT+SpiceDB | yes | submit county dataset |
| POST /v1/coordination/incidents | 3 accredited | JWT+SpiceDB | yes | report coordination incident |
| PUT /v1/coordination/incidents/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | resolve incident |
| GET /v1/coordination/incidents | 3 accredited | JWT+SpiceDB | n/a | list incidents |
| POST /v1/auth/saml/acs | internal gRPC | SVID mTLS | yes | SAML ACS endpoint for county IdPs |

OpenAPI: packages/openapi-contracts/governance/county-national-coordination.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.coordination_incident_raised.v1 | emits | eel.governance.coordination.v1 | operational | no | at-least-once idempotent |
| eel.governance.mou_signed.v1 | emits | eel.governance.coordination.v1 | operational | no | at-least-once idempotent |
| eel.ecc.alert_raised.v1 | consumes | eel.ecc.alert.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| IDMS SAML (county IdPs) | idms-saml | integrations/ (tier B, SAML) | yes | circuit breaker; SAML auth fails closed |
| M16 election-command-centre | internal | crates/county-coord/ | yes | advisory; incident escalated to ECC |
| M18 boundary-transparency | internal | crates/county-coord/ | yes | advisory; boundary dataset reference |
| M33 personnel-dispatch | internal | crates/county-coord/ | yes | advisory; personnel incident link |

## ML and AI

none

## Dependencies

- Modules: M16 (ECC escalation), M18 (boundary reference), M33 (personnel)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

1. FR-M56-001: Create MoU with county and ministry parties.
2. FR-M56-002: Dual-sign MoU with county and national officer signatures.
3. FR-M56-003: Track MoU expiry and alert on approaching renewal.
4. FR-M56-004: Submit and approve county datasets (register, boundary, facility data).
5. FR-M56-005: SAML brokering for heterogeneous county government IdPs.
6. FR-M56-006: Fail closed on SAML authentication failure.
7. FR-M56-007: Report coordination incident with severity and county code.
8. FR-M56-008: Escalate high-severity incidents to ECC alert (M16).
9. FR-M56-009: Resolve incident with resolution notes.
10. FR-M56-010: County portal accessible under county subdomain with county-scoped RBAC.
11. FR-M56-011: Ministry approval tracking for cross-ministry MoUs.
12. FR-M56-012: Admin list of incidents with filter by county, severity, status.
13. FR-M56-013: Audit trail of all MoU and incident transitions.
14. FR-M56-014: i18n on county portal in 11 languages.
15. FR-M56-015: WCAG 2.2 AA on county portal.

## Acceptance criteria

1. AC-M56-01: SAML ACS fails closed on IdP unavailability; no partial auth; test.
2. AC-M56-02: High-severity incident escalated to ECC within 60 s; integration test.
3. AC-M56-03: MoU dual-sign requires two distinct officers; SpiceDB policy test.
4. AC-M56-04: County dataset scoped to requesting county; no cross-county data leak; test.
5. AC-M56-05: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Identity:**
- EC-IDN-09: dual-control MoU signer revoked mid-flow; re-validate both at commit.
- EC-IDN-10: SpiceDB down; fail closed writes.

**Security:**
- EC-SEC-05: injection on MoU content or dataset URL fields.
- EC-SEC-08: XML/SAML XXE; DTD disabled in SAML parser.

**Connectivity:**
- EC-CON-05: flapping county SAML IdP; backoff; fail closed.

**Human:**
- EC-HUM-08: 11 languages; no truncation.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | MoU FSM, incident severity routing, SAML ACS validation | nextest | M56-UNIT- |
| property | County scoping invariant; dual-sign SoD | proptest | M56-PROP- |
| contract | SAML IdP double; M16 ECC alert | Pact | M56-PACT- |
| integration | Full MoU lifecycle; SAML auth; incident escalation | nextest + testcontainers | M56-INT- |
| e2e | County portal; admin MoU and incident flows | Playwright | M56-E2E- |
| security | ZAP; SAML XXE; log scanner | ZAP + Semgrep | M56-SEC- |
| i18n | 11 locales | custom | M56-I18N- |
| a11y | axe on county portal | axe-core | M56-A11Y- |

Tags: `#[trace("FR-M56-006","AC-M56-01","EC-SEC-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with coordination section)
- docs/runbooks/county-saml-broker-failure.md (new)
- platform/observability/governance-svc/coordination-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Heterogeneous county IdP metadata: not all 47 counties operate the same SAML provider; onboarding schedule and metadata update process to be defined.
- County subdomain routing strategy (DNS vs. path-prefix) for the county portal.
