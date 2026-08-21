# M53 voter-id-card

| field | value |
|---|---|
| M-number | M53 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | XL (flagged) |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M53-voter-id-card.md |

## Purpose

Feature-flagged module for voter ID card issuance: manages card batch production orders to the NRB card manufacturer, collection records, blacklist maintenance, and card deactivation lifecycle integrated with the BVR voter register.

## Surfaces and languages

| surface | technology | path under services/bvr-svc/ |
|---|---|---|
| Public card collection status | Astro / Django | astro/bvr-public/ |
| Admin card batch management | Django | django/bvr-admin/ |
| Internal API | Rust (axum) | crates/voter-card/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| VoterCard | id, voter_id, card_number, batch_id, status (ordered/printed/dispatched/collected/deactivated/blacklisted), issued_at | B | T1-consensus |
| CardBatch | id, election_id, constituency_id, order_count, manufacturer_ref, status, ordered_at, received_at | C | T1-consensus |
| CollectionRecord | id, card_id, voter_id, collected_at, station_id, officer_id | B | T1-audit |
| BlacklistEntry | id, card_number, reason, blacklisted_at, blacklist_source | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/voter-cards/status | 2 live-per-user | JWT | n/a | voter's own card status |
| POST /v1/voter-cards/batches | 3 accredited | JWT+SpiceDB | yes | create card batch order |
| GET /v1/voter-cards/batches/{id} | 3 accredited | JWT+SpiceDB | n/a | batch status |
| POST /v1/voter-cards/collections | 3 accredited | JWT+SpiceDB | yes | record card collection |
| POST /v1/voter-cards/blacklist | 3 accredited | JWT+SpiceDB | yes | add card to blacklist |
| DELETE /v1/voter-cards/blacklist/{card_number} | 3 accredited | JWT+SpiceDB | yes | remove from blacklist |
| GET /v1/voter-cards/blacklist | 3 accredited | JWT+SpiceDB | n/a | blacklist query (BVI integration) |
| PUT /v1/voter-cards/{id}/deactivate | 3 accredited | JWT+SpiceDB | yes | deactivate card |
| GET /v1/voter-cards/blacklist/export | 4 bulk | JWT+SpiceDB | n/a | blacklist export for BVI kits |

OpenAPI: packages/openapi-contracts/bvr/voter-id-card.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.voter_card_issued.v1 | emits | eel.bvr.voter_card.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_card_blacklisted.v1 | emits | eel.bvr.voter_card.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_card_deactivated.v1 | emits | eel.bvr.voter_card.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_register_snapshot_ready.v1 | consumes | eel.bvr.voter_register.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| NRB card manufacturer | card-manufacturer | integrations/card-manufacturer/ | yes | circuit breaker; order queued |
| M06 BVI backend (blacklist sync) | internal | crates/voter-card/ | yes | advisory; BVI kits poll blacklist |
| M04 VPRS (pre-registration link) | internal | crates/voter-card/ | yes | advisory |
| M02 BVR backend (voter register) | internal | crates/voter-card/ | yes | hard dependency for voter validation |

## ML and AI

none

## Dependencies

- Modules: M02 (voter register), M04 (pre-registration), M06 (BVI blacklist sync)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py

## Feature checklist

1. FR-M53-001: Feature flag gates entire module; disabled by default; enabled via Commission resolution.
2. FR-M53-002: Create card batch order linked to constituency and election.
3. FR-M53-003: Submit batch order to NRB card manufacturer API.
4. FR-M53-004: Track batch status through manufacturer lifecycle (ordered/printed/dispatched/received).
5. FR-M53-005: Validate voter exists in M02 register before card assignment.
6. FR-M53-006: Assign card number to voter idempotently.
7. FR-M53-007: Record card collection at station with officer ID.
8. FR-M53-008: Prevent duplicate collection for same card.
9. FR-M53-009: Add card to blacklist with reason and source.
10. FR-M53-010: Remove card from blacklist with dual approval and audit reason.
11. FR-M53-011: Blacklist propagated to BVI kits via export endpoint within 30 min.
12. FR-M53-012: Deactivate card on voter deregistration or death notification.
13. FR-M53-013: Public status portal for voter to check their card status using voter ID.
14. FR-M53-014: Admin batch list with filter by constituency, status, election.
15. FR-M53-015: Admin card list with filter by batch, status.
16. FR-M53-016: Audit trail of all card status transitions.
17. FR-M53-017: Export collection records as CSV for reconciliation.
18. FR-M53-018: Alert on batch not received within SLA after dispatch.
19. FR-M53-019: i18n on public status portal in 11 languages.
20. FR-M53-020: WCAG 2.2 AA on public surfaces.
21. FR-M53-021: PII minimisation: card number hashed in logs; voter ID not in analytics events.
22. FR-M53-022: Blacklist export signed; BVI kits verify signature before applying.

## Acceptance criteria

1. AC-M53-01: Feature flag disables all card endpoints when inactive; test.
2. AC-M53-02: Duplicate collection for same card rejected; idempotency test.
3. AC-M53-03: Blacklist export signed and verifiable by BVI; contract test.
4. AC-M53-04: Blacklist propagated to BVI within 30 min of blacklisting; integration test.
5. AC-M53-05: Batch order idempotent on manufacturer reference; test.
6. AC-M53-06: Voter not in M02 register rejected with 422; test.
7. AC-M53-07: PII (voter ID) not present in log output; log-scanner test.
8. AC-M53-08: API p99 latency within routine 99.5% SLA.

## Applicable edge cases

**Data:**
- EC-DAT-01: duplicate collection record submission; idempotent.
- EC-DAT-13: deactivated card in search; tombstone honoured.

**Security:**
- EC-SEC-10: voter enumeration via card status endpoint; constant-time shape; step-up.
- EC-SEC-16: voter ID and card number not in logs.
- EC-SEC-05: injection on card number or voter ID fields.

**Human:**
- EC-HUM-08: 11 languages on public portal; no truncation.

**Connectivity:**
- EC-CON-05: flapping manufacturer API; backoff; retry budget.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Card FSM, feature flag gate, blacklist logic | nextest | M53-UNIT- |
| property | Collection idempotency; blacklist propagation timing | proptest | M53-PROP- |
| contract | Manufacturer double; M06 BVI blacklist export | Pact | M53-PACT- |
| integration | Full card lifecycle with testcontainers; blacklist sync | nextest + testcontainers | M53-INT- |
| e2e | Public status portal; admin batch and collection | Playwright | M53-E2E- |
| security | ZAP; log scanner; enumeration timing | ZAP + Semgrep | M53-SEC- |
| i18n | 11 locales | custom | M53-I18N- |
| a11y | axe on public portal | axe-core | M53-A11Y- |

Tags: `#[trace("FR-M53-001","AC-M53-01","EC-SEC-10")]`

## Runbooks and dashboards

- docs/runbooks/bvr-svc-degradation.md (extend with voter-card section)
- docs/runbooks/voter-card-blacklist-sync-failure.md (new)
- platform/observability/bvr-svc/voter-card-dashboard.json
- platform/observability/bvr-svc/slo-alerts.yaml

## Open questions

- Scope activation: Commission resolution and legal basis for voter ID cards not yet confirmed; module remains feature-flagged until decision.
- NRB card manufacturer API specification and SLA not received; double must gate development until live credentials available.
