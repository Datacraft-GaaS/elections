# M64 voter-list-public-inspection

| field | value |
|---|---|
| M-number | M64 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-5.7 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M64-voter-list-public-inspection.md |

## Purpose

Publishes verifiable roll snapshots for the statutory public inspection window, accepts voter objections and correction claims with biometric claimant verification, and auto-refers disputed claims to eCourt at the T-60 deadline.

## Surfaces and languages

| surface | technology | path under services/portal-svc/ |
|---|---|---|
| Public voter inspection portal | Astro | astro/inspection/ |
| Admin inspection management | Django | django/portal-admin/ |
| Internal API | Rust (axum) | crates/inspection/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| RollSnapshot | id, election_id, snapshot_date, voter_count, merkle_root, content_url, signed_at | C | T1-consensus |
| Objection | id, snapshot_id, objector_id_hash, subject_voter_id_hash, grounds, status (pending/upheld/dismissed), filed_at | B | T1-consensus |
| CorrectionClaim | id, snapshot_id, claimant_id_hash, biometric_verify_ref, field_to_correct, proposed_value, status (pending/verified/applied/rejected), filed_at | B | T1-consensus |
| AutoReferralPacket | id, claim_id, ecourt_ref, referred_at, idempotent_key | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/inspection/snapshots | 1 signed-static | public | n/a | list roll snapshots |
| GET /v1/inspection/snapshots/{id} | 1 signed-static | public | n/a | snapshot metadata and Merkle root |
| GET /v1/inspection/snapshots/{id}/verify | 2 live-per-user | public | n/a | voter self-check (anti-enumeration guards) |
| POST /v1/inspection/objections | 2 live-per-user | JWT | yes | file objection |
| GET /v1/inspection/objections/{id} | 2 live-per-user | JWT | n/a | objection status |
| POST /v1/inspection/corrections | 2 live-per-user | JWT | yes | file correction claim |
| GET /v1/inspection/corrections/{id} | 2 live-per-user | JWT | n/a | correction status |
| PUT /v1/inspection/objections/{id}/decide | 3 accredited | JWT+SpiceDB | yes | adjudicate objection |
| PUT /v1/inspection/corrections/{id}/apply | 3 accredited | JWT+SpiceDB | yes | apply correction to register |
| POST /v1/inspection/corrections/{id}/refer | 3 accredited | JWT+SpiceDB | yes | manual eCourt referral |

OpenAPI: packages/openapi-contracts/portal/voter-list-public-inspection.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.portal.objection_filed.v1 | emits | eel.portal.inspection.v1 | operational | no | at-least-once idempotent |
| eel.portal.correction_applied.v1 | emits | eel.portal.inspection.v1 | operational | no | at-least-once idempotent |
| eel.portal.auto_referral_sent.v1 | emits | eel.portal.inspection.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_register_snapshot_ready.v1 | consumes | eel.bvr.voter_register.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M04 VPRS (voter self-service link) | internal | crates/inspection/ | yes | advisory |
| M01 BVR device (biometric claimant verify) | internal | crates/inspection/ | yes | advisory; correction claim deferred if BVR unavailable |
| M10 rts-backend (snapshot source) | internal | crates/inspection/ | yes | advisory |
| eCourt (auto-referral) | ecourt | integrations/ecourt/ | yes | circuit breaker; referral queued; alert if queue grows |

## ML and AI

none

## Dependencies

- Modules: M04 (voter link), M01 (biometric verify), M10 (snapshot source)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-astro, eel-web-py

## Feature checklist

1. FR-M64-001: Publish roll snapshot with Merkle root and verifiable hash on inspection window open.
2. FR-M64-002: Public voter self-check: confirm own registration without revealing other voters (anti-enumeration).
3. FR-M64-003: Anti-enumeration: constant-time response shape; rate-limit per IP /64; step-up on suspicious volume.
4. FR-M64-004: Accept voter objection against another voter's registration with grounds.
5. FR-M64-005: Accept correction claim for own registration details with biometric verification reference.
6. FR-M64-006: Adjudicate objection (upheld/dismissed) with reason.
7. FR-M64-007: Apply upheld correction to M02 voter register.
8. FR-M64-008: Auto-refer pending claims to eCourt idempotently at T-60 deadline.
9. FR-M64-009: Idempotent auto-referral: double-fire at T-60 produces one eCourt packet (EC-LEG-09).
10. FR-M64-010: Public portal shows snapshot metadata and Merkle root; content via CDN.
11. FR-M64-011: Admin list of objections and corrections with filter by status and county.
12. FR-M64-012: Admin can close inspection window; no new filings after close.
13. FR-M64-013: Snapshot Merkle root verifiable by any voter using inclusion proof.
14. FR-M64-014: i18n in 11 languages on public portal.
15. FR-M64-015: WCAG 2.2 AA on public surfaces.
16. FR-M64-016: All PII in claims stored under data-class B; hashed in logs.

## Acceptance criteria

1. AC-M64-01: Voter enumeration prevented: timing oracle test p95 delta <5 ms between found/not-found responses (EC-SEC-10).
2. AC-M64-02: Auto-referral at T-60 idempotent; double-fire test produces single eCourt packet (EC-LEG-09).
3. AC-M64-03: Merkle root verifiable against snapshot content; inclusion proof test.
4. AC-M64-04: Objection and correction filing blocked after inspection window close; test.
5. AC-M64-05: PII (voter ID hash) not present in plain in logs; log-scanner test.
6. AC-M64-06: Biometric claimant verify required before correction claim proceeds; integration test.
7. AC-M64-07: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Security:**
- EC-SEC-10: voter enumeration via self-check; constant-time shape; step-up threshold.
- EC-SEC-12: cache poisoning of roll snapshots; key includes signature version.
- EC-SEC-16: voter ID hash not in plain in logs.

**Legal:**
- EC-LEG-09: T-60 auto-referral race; single referral; idempotent scheduler.
- EC-TIM-09: register freeze race T-60/T-30; OPA hard-close at tx time.
- EC-TIM-08: inspection window close; boundary test +-1 s.

**Data:**
- EC-DAT-01: duplicate objection filing; idempotent.
- EC-DAT-13: soft-deleted voter in snapshot search; tombstone honoured.

**Scale:**
- EC-SCL-01: 5M visitors/hr on public portal; 95% edge.
- EC-SCL-02: thundering herd on snapshot STH; ETag 304.

**Human:**
- EC-HUM-08: 11 languages; long strings in objection grounds.
- EC-HUM-10: low-literacy path: USSD self-check fallback.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Inspection window FSM, anti-enumeration guard, auto-referral idempotency | nextest | M64-UNIT- |
| property | Merkle root verification; objection idempotency; T-60 race | proptest | M64-PROP- |
| contract | eCourt referral double; M02 correction apply; M01 biometric verify | Pact | M64-PACT- |
| integration | Full inspection lifecycle; auto-referral; window close | nextest + testcontainers | M64-INT- |
| e2e | Public self-check; objection and correction filing; admin adjudicate | Playwright | M64-E2E- |
| load | 5M/hr public portal | k6 + Gatling | M64-LOAD- |
| security | Timing oracle ≤5 ms delta; ZAP; log scanner | ZAP + Semgrep | M64-SEC- |
| i18n | 11 locales | custom | M64-I18N- |
| a11y | axe on public surfaces | axe-core | M64-A11Y- |

Tags: `#[trace("FR-M64-008","AC-M64-02","EC-LEG-09")]`

## Runbooks and dashboards

- docs/runbooks/portal-svc-degradation.md (extend with inspection section)
- docs/runbooks/ecourt-referral-queue.md (new; trigger when queue exceeds threshold)
- platform/observability/portal-svc/inspection-dashboard.json
- platform/observability/portal-svc/slo-alerts.yaml

## Open questions

- eCourt API specification not yet available; double gates development.
- Whether biometric verification for correction claims requires the claimant to be physically present at an IEBC office or can be remote; affects M01 integration scope.
