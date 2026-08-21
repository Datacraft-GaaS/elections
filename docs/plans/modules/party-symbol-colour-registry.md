# M67 party-symbol-colour-registry

| field | value |
|---|---|
| M-number | M67 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M67-party-symbol-colour-registry.md |

## Purpose

Manages political party symbol submissions and colour registrations, runs perceptual hash (pHash) and CIE Delta-E similarity checks to prevent near-threshold disputes, produces ballot design packages for the printer, and issues certificates of symbol registration.

## Surfaces and languages

| surface | technology | path under services/crms-svc/ |
|---|---|---|
| Public symbol registry | Astro | astro/crms-public/ |
| Party submission portal | Elixir (Phoenix LiveView) | ex/party_portal/ |
| Admin adjudication console | Django | django/crms-admin/ |
| Internal API | Rust (axum) | crates/crms-api/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| SymbolSubmission | id, party_id, election_id, image_url, phash, status (submitted/similarity-check/approved/rejected/disputed), submitted_at | C | T1-consensus |
| ColourRegistration | id, party_id, election_id, primary_hex, secondary_hex, pantone_ref, delta_e_distances (JSONB with other parties), status, registered_at | C | T1-consensus |
| BallotDesignPackage | id, party_id, election_id, symbol_url, colour_profile_url, printer_spec_url, signed_at, version | C | T1-consensus |
| SymbolDispute | id, symbol_submission_id, disputing_party_id, dispute_grounds, adjudication, resolved_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/symbols/registry | 1 signed-static | public | n/a | public symbol registry |
| POST /v1/symbols/submissions | 3 accredited | JWT+SpiceDB | yes | submit symbol |
| GET /v1/symbols/submissions/{id} | 3 accredited | JWT+SpiceDB | n/a | submission status |
| POST /v1/symbols/submissions/{id}/similarity-check | internal gRPC | SVID mTLS | yes | trigger pHash + ΔE check |
| PUT /v1/symbols/submissions/{id}/approve | 3 accredited | JWT+SpiceDB | yes | approve symbol |
| PUT /v1/symbols/submissions/{id}/reject | 3 accredited | JWT+SpiceDB | yes | reject with reason |
| POST /v1/symbols/disputes | 3 accredited | JWT+SpiceDB | yes | file symbol dispute |
| PUT /v1/symbols/disputes/{id}/adjudicate | 3 accredited | JWT+SpiceDB | yes | adjudicate dispute |
| POST /v1/colours/registrations | 3 accredited | JWT+SpiceDB | yes | register party colours |
| GET /v1/colours/registrations/{party_id} | 3 accredited | JWT+SpiceDB | n/a | colour registration |
| POST /v1/ballot-design/{party_id} | 3 accredited | JWT+SpiceDB | yes | generate ballot design package |
| GET /v1/ballot-design/{party_id} | 3 accredited | JWT+SpiceDB | n/a | download ballot design package |

OpenAPI: packages/openapi-contracts/crms/party-symbol-colour-registry.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.crms.symbol_approved.v1 | emits | eel.crms.symbol.v1 | operational | no | at-least-once idempotent |
| eel.crms.symbol_rejected.v1 | emits | eel.crms.symbol.v1 | operational | no | at-least-once idempotent |
| eel.crms.ballot_design_issued.v1 | emits | eel.crms.symbol.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M45 party-registration | internal | crates/crms-api/ | yes | hard dependency; party must be registered |
| M12 CRMS candidate | internal | crates/crms-api/ | yes | advisory; ballot design used in nomination |
| Printer / Pantone | pantone | integrations/pantone/ | yes | circuit breaker; manual Pantone entry fallback |

## ML and AI

- pHash perceptual similarity check (Rust crate, on-service): threshold ≥99% match on symbol set tested in WP-4.11.
- CIE Delta-E colour distance computation (Rust): ΔE registered per party pair in ColourRegistration.

## Dependencies

- Modules: M45 (party existence), M12 (nomination ballot design)
- Platform: P01 (SpiceDB), P04 (Kafka outbox), MinIO (symbol images)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

1. FR-M67-001: Submit party symbol image (SVG/PNG); store in MinIO.
2. FR-M67-002: Run pHash similarity check against all registered symbols; flag matches above threshold.
3. FR-M67-003: Run CIE Delta-E check against all registered colour pairs; flag near-threshold pairs.
4. FR-M67-004: Require party to be registered in M45 before symbol submission.
5. FR-M67-005: Admin adjudicates similarity disputes with reason.
6. FR-M67-006: Approve symbol; emit symbol_approved event.
7. FR-M67-007: Reject symbol with reason; party can resubmit.
8. FR-M67-008: Register primary and secondary colours with Pantone reference.
9. FR-M67-009: Compute and store Delta-E distances between all party colour pairs.
10. FR-M67-010: Alert when new registration brings Delta-E below minimum distance.
11. FR-M67-011: Generate ballot design package with symbol, colour profile, and printer spec.
12. FR-M67-012: Sign ballot design package; version-stamped.
13. FR-M67-013: Issue certificate of symbol registration; HSM-signed.
14. FR-M67-014: Party portal shows submission status and any similarity flags.
15. FR-M67-015: Public registry lists all approved symbols and colours.
16. FR-M67-016: Admin list of submissions with filter by status and election.
17. FR-M67-017: Dispute filing and adjudication workflow with audit trail.
18. FR-M67-018: pHash and ΔE thresholds configurable without code deploy.
19. FR-M67-019: i18n on public registry in 11 languages.
20. FR-M67-020: WCAG 2.2 AA; colour accessibility: ΔE registry used for accessibility check too (EC-HUM-09).

## Acceptance criteria

1. AC-M67-01: pHash similarity ≥99% detected on full symbol set; unit test corpus (WP-4.11 done criterion).
2. AC-M67-02: Delta-E computed correctly; near-threshold alert fires at configured minimum; unit test.
3. AC-M67-03: Ballot design package signed; version-stamped; idempotent regeneration.
4. AC-M67-04: Submission blocked if party not in M45; test.
5. AC-M67-05: Dispute adjudication idempotent; double-fire test.
6. AC-M67-06: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Human:**
- EC-HUM-09: colour-blind symbols and colours; ΔE registry used for accessibility check; simulation test.

**Security:**
- EC-SEC-07: path traversal on symbol image upload; sanitised.
- EC-SEC-05: injection on colour hex or Pantone ref fields.

**Data:**
- EC-DAT-01: duplicate symbol submission; idempotent.
- EC-DAT-13: rejected symbol in public registry; tombstone honoured; not shown.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | pHash similarity, Delta-E computation, submission FSM | nextest | M67-UNIT- |
| property | pHash threshold at ±1 pixel; Delta-E boundary | proptest | M67-PROP- |
| contract | M45 party existence; printer Pantone double | Pact | M67-PACT- |
| integration | Full symbol lifecycle; colour registration; ballot design | nextest + testcontainers | M67-INT- |
| e2e | Party portal submission; admin adjudicate; public registry | Playwright | M67-E2E- |
| security | ZAP; path traversal on upload; log scanner | ZAP + Semgrep | M67-SEC- |
| i18n | 11 locales | custom | M67-I18N- |
| a11y | axe; colour-blindness simulation (EC-HUM-09) | axe-core + simulation | M67-A11Y- |

Tags: `#[trace("FR-M67-002","AC-M67-01","EC-HUM-09")]`

## Runbooks and dashboards

- docs/runbooks/crms-svc-degradation.md (extend with symbol-registry section)
- platform/observability/crms-svc/symbol-registry-dashboard.json
- platform/observability/crms-svc/slo-alerts.yaml

## Open questions

- Pantone API access and licensing for digital Delta-E reference values; manual entry fallback must be ready for go-live.
- Minimum Delta-E threshold is currently not specified in the spec; requires agreement between IEBC design team and accessibility auditors.
