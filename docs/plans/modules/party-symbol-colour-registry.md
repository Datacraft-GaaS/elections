# M67 party-symbol-colour-registry

| field | value |
|---|---|
| M-number | M67 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M67-party-symbol-colour-registry.md |

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

Symbol submission:

1. FR-M67-001: Accept party symbol as SVG (max 5 MB) or PNG (min 300 dpi, max 10 MB); reject files below threshold with actionable error message. (FR-67.01)
2. FR-M67-002: Validate SVG structure for embedded scripts and external references; reject any SVG with DOCTYPE, script elements, or href pointing outside the allowlist. (FR-67.01)
3. FR-M67-003: Generate normalised 512x512 bitmap from the submitted symbol; store alongside original for comparison baseline. (FR-67.01)
4. FR-M67-004: Compute 64-bit perceptual hash (pHash) on the normalised bitmap at four rotation angles (0, 90, 180, 270 degrees); store all four values per submission. (FR-67.02)
5. FR-M67-005: Run Tanimoto similarity coefficient on pixel-presence bitmask against every registered symbol in the current cycle; flag any pair where pHash Hamming distance is less than the configured threshold (default 10). (FR-67.02)
6. FR-M67-006: Return similarity report including conflicting party identifiers, individual scores, and side-by-side image URLs within 2 s of upload completion. (FR-67.02 / NFR-67.02)
7. FR-M67-007: Accept logotype (wordmark) as SVG or high-res PNG; store separately from symbol; no similarity check applied to logotype. (FR-67.04)
8. FR-M67-008: Require party to be registered in M45 before any symbol submission is accepted; return 403 with eel_error_code: party_not_registered otherwise.

Colour registration:

9. FR-M67-009: Accept up to three colours per party (roles: primary, secondary, accent); require Pantone PMS code, RGB triple, CMYK percentages, and HEX; RAL code optional. (FR-67.03)
10. FR-M67-010: Map submitted Pantone PMS code to RGB and CMYK via offline ICC profile database; emit a warning when submitted RGB deviates by more than 5 delta-E from profile value. (FR-67.03 / NFR-67.06)
11. FR-M67-011: Compute CIE76 delta-E between each newly registered primary colour and every other registered primary colour; alert the officer when any pair falls below the configured minimum (default delta-E 10). (FR-67.03)
12. FR-M67-012: Store per-pair delta-E distances as JSONB in ColourRegistration; expose via registration detail endpoint.
13. FR-M67-013: Report WCAG contrast ratio (against white and black) for every registered colour; flag any primary colour failing the 4.5:1 AA ratio. (NFR-67.01)
14. FR-M67-014: Validate CMYK values against Pantone ICC colour bridge; flag if delta-E deviation exceeds 5; M12 receives validated CMYK values only. (NFR-67.06)

Approval and dispute workflow:

15. FR-M67-015: Registrar Officer reviews submissions in the admin console; decision options: approved, rejected_similarity, rejected_incomplete, deferred_pending_info. (FR-67.05)
16. FR-M67-016: Approved record is signed with officer JWT; approval timestamp and officer UUID stored and included in the hash chain entry. (FR-67.05)
17. FR-M67-017: Rejection triggers party notification via M45 communication channel with rejection reason code and human-readable narrative. (FR-67.05)
18. FR-M67-018: Officer may override a similarity conflict with a written determination; override recorded with officer UUID and narrative; visible in IAB audit view. (FR-67.02 / FR-67.05)
19. FR-M67-019: Dispute filing workflow: disputing party submits dispute grounds; officer adjudicates with determination text; full audit trail retained in T1-audit store.
20. FR-M67-020: All registry records are append-only and hash-chained; any attempt to modify an approved record is rejected with 405. (NFR-67.04)
21. FR-M67-021: pHash Hamming threshold and CIE76 delta-E minimum threshold are configurable via runtime config (environment variable or feature-flag store) without code deploy.

Rebrand workflow:

22. FR-M67-022: Party submits rebrand request with new symbol and colours as a draft; existing approved registration remains active until the rebrand is approved. (FR-67.06)
23. FR-M67-023: Rebrand subject to the same pHash and delta-E checks against all other parties; similarity report returned before officer review. (FR-67.06)
24. FR-M67-024: Rebrand requires dual officer sign-off (Registrar Officer and Deputy Registrar); single-officer approval returns 422. (FR-67.06)
25. FR-M67-025: On rebrand approval, archive old record as immutable with archived status; activate new record; emit eel.crms.symbol_approved.v1 and notify M12 via webhook within 60 s. (FR-67.06)
26. FR-M67-026: Any new symbol registration or rebrand request received after T-90 is blocked with 423 eel_error_code: registry_frozen unless the request carries a Commissioner countersignature. (FR-67.10)

Ballot design and print export:

27. FR-M67-027: Generate versioned ballot design package per party and cycle: approved SVG symbol, CMYK primary colour, RGB primary colour, logotype URL, and party name. (FR-67.07)
28. FR-M67-028: Expose ballot design package via authenticated API; M12 CRMS and printer consume by party_id, cycle, and explicit version; package generation is idempotent. (FR-67.07)
29. FR-M67-029: At T-90, lock the cycle registry snapshot; deliver signed manifest (SHA-256 over all approved symbol hashes) to M12 within 1 hour. (FR-67.10)
30. FR-M67-030: Any post-T-90 Commissioner-approved rebrand triggers re-delivery of updated ballot design package to M12 within 1 hour; package version increments.

Certificates and public registry:

31. FR-M67-031: Generate on-demand signed PDF certificate per party; content: symbol hash, colours, approval date, officer signature; delivered within 30 s; HSM-signed via P06. (FR-67.08)
32. FR-M67-032: Public registry browser lists all approved symbols with colour swatches, HEX codes, and Pantone names; symbol images downloadable as PNG at 300 dpi. (FR-67.09)
33. FR-M67-033: Public registry and submission portal meet WCAG 2.2 AA; every symbol image carries alt text; every colour swatch carries a text colour name; accessibility report published with registry. (NFR-67.01)

Internationalisation and observability:

34. FR-M67-034: Public registry rendered in English and Swahili; all user-visible error messages include an eel_error_code field for programmatic handling.
35. FR-M67-035: All state transitions (submitted, approved, rejected, archived, rebranded) emit audit events to P05 carrying eel_submission_id, eel_party_id, eel_officer_id, and eel_cycle fields.
36. FR-M67-036: OpenTelemetry traces exported to central collector; module Grafana dashboard and Alertmanager ruleset present before OAT gate (NFR-M67-MONITORING).

## Acceptance criteria

1. AC-M67-01: pHash similarity check completes at p99 <= 2 s on a registry of 200+ parties; measured by automated benchmark against a synthesised symbol corpus (WP-4.11 done criterion).
2. AC-M67-02: Tanimoto conflict detection rate >= 99% on a labelled corpus of 200 known-similar symbol pairs verified at four rotation angles.
3. AC-M67-03: CIE76 delta-E computation deviates by < 0.01 from reference values; near-threshold alert fires for every pair below the configured minimum (default 10); covered by unit test with 50 colour pairs.
4. AC-M67-04: Public registry API (GET /v1/symbols/registry) p99 <= 200 ms at 1000 rpm steady-state load on a registry of 200 parties; service availability class: routine 99.5%.
5. AC-M67-05: Ballot design package generation is idempotent; repeated calls for the same party_id and cycle return the same version number and identical binary content; package carries a valid HSM signature verifiable by P06 public key.
6. AC-M67-06: All registered party ballot design packages delivered to M12 before T-90 snapshot; delivery confirmed by signed manifest receipt.
7. AC-M67-07: Signed PDF certificate issued within 30 s of request for any approved party; certificate signature verified against P06 public key in automated integration test.
8. AC-M67-08: Symbol submission or colour registration blocked when the requesting party is absent from M45; response is 403 with body containing eel_error_code: party_not_registered.
9. AC-M67-09: Dispute adjudication write is idempotent; sending the same adjudication request twice returns 200 with identical payload and zero additional audit entries.
10. AC-M67-10: Post-T-90 submission or rebrand without Commissioner countersignature returns 423 with eel_error_code: registry_frozen; verified by integration test targeting the freeze endpoint.
11. AC-M67-11: Public registry passes automated axe-core scan with zero critical or serious WCAG 2.2 AA violations; manual screen-reader walkthrough completes all core tasks without assistive-technology failure.
12. AC-M67-12: All approved registry records are append-only; a direct PATCH or DELETE on an approved symbol record returns 405; verified by integration test.
13. AC-M67-13: CMYK values in every ballot design package deviate by <= 5 delta-E from the Pantone ICC profile reference for that Pantone code; verified against five reference Pantone colours.
14. AC-M67-14: RTO for the symbol registry service <= 1 hour; RPO <= 5 minutes; demonstrated via a documented PITR drill in the runbook before OAT gate.
15. AC-M67-15: Secrets (JWTs, HSM key handles, Pantone API tokens) produce zero hits in the log-scanner fixture (Tier A/B patterns); CI gate blocks merge on any hit (EC-SEC-16).

## Applicable edge cases

EC-DAT-01: duplicate symbol submission with same party_id and file hash returns cached 202 with no re-execution of pHash pipeline.
EC-DAT-02: duplicate submission idempotency key reused with a different SVG body returns 422 eel_error_code: idempotency_key_reuse, preventing silent substitution.
EC-DAT-06: ColourRegistration schema v1 (no delta_e_distances field) and v2 coexist during migration; both accepted by the API; dual-emit ensures downstream consumers see correct shape.
EC-DAT-07: partial batch failure during bulk symbol migration (e.g. import of all parties from M45 seed); per-row error report produced; valid rows committed; failed rows re-queued.
EC-DAT-09: party names containing Unicode characters (Luhya, Somali, Arabic) are stored as NFC-normalised; public registry sorts correctly via ICU collation; no grapheme truncation in PDF certificate.
EC-DAT-13: rejected or archived symbol records are excluded from the public registry response; tombstone is honoured in search; purge job does not remove audit entries.
EC-DAT-14: submitted SVG exceeds 5 MB or PNG exceeds 10 MB; upload rejected at the multipart boundary with 413 and eel_error_code: file_too_large before pHash pipeline is invoked.
EC-SEC-05: injection attempted via colour hex string, Pantone ref field, description text, or party name; all inputs are parameterised; Semgrep and ZAP scans confirm no raw interpolation.
EC-SEC-06: SVG containing an external href or xlink:href to a private-range IP (SSRF vector); SVG sanitiser strips all external references before storage; allowlist enforced.
EC-SEC-07: path traversal attempt in multipart filename (e.g. ../../etc/passwd.svg); filename sanitised to UUID-keyed object path before MinIO PUT; fuzz corpus covers ../ sequences.
EC-SEC-08: SVG with embedded DOCTYPE or ENTITY declaration (XXE); parser configured with DTD disabled; billion-laughs fuzz case returns 400.
EC-SEC-12: CDN cache poisoning of the public registry snapshot; cache key includes signature version; client SDK verifies manifest hash before rendering; stale signed copy fails verification.
EC-SEC-16: officer JWT, HSM key handle, and Pantone API token must not appear in OpenTelemetry traces or application logs; log-scanner fixture in CI enforces zero Tier A/B matches.
EC-IDN-02: JWT expires mid-symbol submission or mid-approval review; in-flight request completes using the existing token; the next request receives 401 and the portal silently refreshes.
EC-IDN-09: dual-control approver for a rebrand has their account revoked between the first and second sign-off; the pending sign-off is invalidated; a new approver must be assigned.
EC-IDN-10: SpiceDB unavailable during an adjudication write; the write fails closed with 503 problem+json; read-only public registry remains available via class-C cached read.
EC-HUM-06: screen reader users complete symbol submission and adjudication workflows without assistive-technology failures; axe-core and manual NVDA walkthrough included in M67-A11Y- suite.
EC-HUM-08: 11 language locales active on the public registry; long party names in Somali or Arabic do not truncate in swatch labels or PDF certificates; ICU plurals applied.
EC-HUM-09: colour-blind simulation (deuteranopia, protanopia, tritanopia) applied to every registered colour swatch; ΔE registry entries used to confirm each party's palette remains visually distinct under simulation.
EC-SCL-01: 5M visitors per hour on public registry at election announcement time; >= 95% of requests served from CDN edge; origin p99 <= 500 ms for cache misses.
EC-SCL-04: consumer lag on eel.crms.symbol.v1 topic exceeds 60 s; Alertmanager pages on-call; backpressure applied; no event loss; verified by slow-consumer integration test.
EC-OPS-08: MinIO bucket quota reached during symbol image upload; service returns 507 with eel_error_code: storage_quota_exceeded; alert fires; no partial or corrupt object written.
EC-TIM-07: s.14A close extension shifts the T-90 freeze boundary; the freeze timestamp is reloaded from the HSM-signed calendar amendment; forged extension without valid HSM signature is rejected.
EC-LEG-02: court-ordered postponement of the election shifts the registration deadline; the system reloads the gazetted deadline from a signed calendar amendment via the geo-fence bundle endpoint; unverified date overrides are rejected.
EC-LEG-09: T-60 auto-referral scheduler fires twice due to a race condition; second referral is idempotent; only one referral record created; double-fire integration test confirms.
EC-CON-05: flapping network link during multipart SVG upload; client retries with the same idempotency key; server deduplicates against the cached response; zero duplicate pHash executions on reconnection.

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
- Minimum Delta-E threshold is currently not specified in the spec; requires agreement between the Commission design team and accessibility auditors.
