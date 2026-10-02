# M07 bvv

| field | value |
|---|---|
| M-number | M07 |
| Home service | bvi-svc (ADR-0020) |
| Owning work package | WP-3.7 (docs/plans/04-wave-3-critical-path.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M07-bvv.md |

## Purpose

Statutory 30-day public inspection of the voter register: any citizen confirms their own entry and files objections against errors or omissions, without exposing the register to bulk enumeration.

## Surfaces and languages

| surface | technology | path under services/bvi-svc/ |
|---|---|---|
| Public web portal + kiosk | Astro on packages/eel-web-astro | astro/bvv-portal/ |
| Objections API | Rust (axum + utoipa) | crates/bvi-api/ (shared, scoped routes) |
| Back-office objections admin | Django on packages/eel-web-py | django/bvv-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| VoterLookupAudit | audit_id, actor_session_hash, target_voter_id_hash, ts, ip_hash, result_status | C | T1-audit |
| Objection | obj_id, objector_session_hash, target_voter_id, type, evidence_ref, status, resolution | B | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/bvi/bvv.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /bvv/v1/lookup | 2 live-per-user | JWT (OTP-verified session) | n/a | Self-lookup by ID + OTP; returns redacted record |
| POST /bvv/v1/objections | 2 live-per-user | JWT | yes | File objection on own or third-party record |
| GET /bvv/v1/objections/{id}/status | 2 live-per-user | JWT | n/a | Track objection status |
| GET /bvv/v1/objections | 3 accredited | JWT+SpiceDB | n/a | the Commission back-office objection queue |
| PATCH /bvv/v1/objections/{id} | 3 accredited | JWT+SpiceDB | yes | Adjudicate objection |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvi.voter_lookup_audited.v1 | emits | eel.bvi.audit.v1 | analytics | yes | at-least-once idempotent |
| eel.bvi.objection_filed.v1 | emits | eel.bvi.objection.v1 | operational | yes | at-least-once idempotent |
| eel.bvi.objection_resolved.v1 | emits | eel.bvi.objection.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.voter_status_changed.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVR back-end (M02) | internal | integrations/bvr-backend/ | yes | register reads cached; degrade to last-known |
| SMS gateway | I08 | integrations/sms-gw/ | yes | OTP retry; email fallback |
| Transparency log (P05) | internal | integrations/tlog/ | yes | audit anchoring queued |

## ML and AI

none

## Dependencies

- Modules: M02 (register reads), M04 (related enumeration risk; share constant-time shim)
- Platform: P04 (IAM/OTP), P05 (audit)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, eel-web-astro, eel-web-py

## Feature checklist

Lookup and display:
1. FR-M07-001: public self-lookup by national ID number + OTP sent to registered mobile. (spec FR-M07-001)
2. FR-M07-002: kiosk self-service with QR-scan or manual ID entry; kiosk session isolated per citizen. (spec FR-M07-002)
3. FR-M07-003: return name, DOB year only, polling station name and code, ward, constituency, county, and registration status. (spec FR-M07-003)
4. FR-M07-004: mask all sensitive fields: biometrics never returned; full DOB masked to year; mobile number starred. (spec FR-M07-004)
5. FR-M07-005: anti-enumeration constant-time response: hit and miss return identical payload structure and size.
6. FR-M07-006: CDN-cached public responses; cache key excludes session data; no PII stored at CDN edge.
7. FR-M07-007: lookup p99 <= 2 s; 100,000 concurrent users supported during peak 30-day inspection window.
8. FR-M07-008: per-session cap of 20 lookups/day enforced server-side; counter resets at midnight EAT.
9. FR-M07-009: voter inspection window enforcement: portal closes outside gazetted 30-day window; countdown and re-open date displayed.
10. FR-M07-010: USSD short-code path: citizen dials code, enters ID, receives SMS reply with polling station name.

Objections:
11. FR-M07-011: file objection on own record: types are rectification, transfer request, and removal request; each creates an audit trail. (spec FR-M07-005)
12. FR-M07-012: file objection on third-party record: justification required; evidence upload accepted (PDF, JPG, PNG, <= 10 MB, malware scan). (spec FR-M07-006)
13. FR-M07-013: objection status tracking: citizen views current status and history via web or SMS. (spec FR-M07-007)
14. FR-M07-014: SMS/email notification sent to objector on every status change.
15. FR-M07-015: back-office objection queue with adjudication workflow, response templates, and SLA countdown timers. (spec FR-M07-008)
16. FR-M07-016: objection SLA: first response to objector <= 48 h; full resolution <= 30 days; both configurable.
17. FR-M07-017: escalation trigger: overdue objection auto-escalates to senior officer with audit entry.

Anti-abuse and audit:
18. FR-M07-018: rate limit per IP, per session, and per NID; CAPTCHA on suspicious patterns; configurable thresholds hot-reloadable. (spec FR-M07-009)
19. FR-M07-019: enumeration-pattern alert: fires within 60 s at configured lookup-rate threshold; IP blocked; M26 anomaly detector notified. (spec FR-M07-009)
20. FR-M07-020: per-lookup audit entry: actor session hash, target voter ID hash, timestamp, IP hash, result status. (spec FR-M07-010)
21. FR-M07-021: every lookup permanently logged; entries not erasable even on data-subject access request (SAR); SAR returns lookup metadata only.
22. FR-M07-022: audit log anchored to transparency log (P05) for tamper-evidence; anchor event emitted per batch.
23. FR-M07-023: ODPC data-subject access request handler: exports citizen's own lookup audit entries on demand.

Kiosk and accessibility:
24. FR-M07-024: kiosk mode auto-clear: session wiped on idle timeout; no PII cached in browser or local storage.
25. FR-M07-025: OTP delivery p95 <= 60 s; email OTP fallback activated after two SMS retries.
26. FR-M07-026: multilingual: English and Kiswahili on all user-facing surfaces; extensible to 11 languages. (spec FR-M07-011)
27. FR-M07-027: WCAG 2.2 AA on all surfaces; full screen-reader support; keyboard-only navigation. (spec FR-M07-012)
28. FR-M07-028: KSL video explainer for objection filing accessible from help panel; captions on all video.
29. FR-M07-029: high-contrast mode and configurable font scaling on web portal.
30. FR-M07-030: low-literacy mode: minimal text, icon + label pairs, <= 3 steps to station name.

Compliance and privacy:
31. FR-M07-031: Kenya DPA 2019 s.30(1)(e) compliance: consent banner, purpose disclosure, and retention notice shown before first lookup. (spec FR-M07-013)
32. FR-M07-032: DPA retention policy enforced: audit log entries purged after statutory retention period; purge job audited.

## Acceptance criteria

AC-M07-01: 100,000 concurrent lookups sustained; p99 <= 2 s at peak (load test).
AC-M07-02: 5M lookups/day load test: CDN-cached responses contain no PII; origin p99 <= 500 ms (load test).
AC-M07-03: bulk-enumeration attempt blocked at configured threshold; IP blocked; alert fired within 60 s (red-team).
AC-M07-04: constant-time anti-enumeration: p95 response-time delta between hit and miss < 5 ms (timing oracle test).
AC-M07-05: ODPC audit rated compliant: consent, purpose, and retention controls evidenced (ODPC review).
AC-M07-06: objection workflow closes within SLA: 48 h first response; 30-day resolution (UAT).
AC-M07-07: per-lookup audit entry present for every request; contains actor hash, target hash, IP hash, timestamp (audit trace).
AC-M07-08: no biometric data or full DOB exposed in any API response or CDN cache entry (automated scan + manual review).
AC-M07-09: portal outside inspection window returns 423 with countdown; no voter data returned (functional test).
AC-M07-10: WCAG 2.2 AA: axe zero violations; screen-reader traversal completes full lookup + objection flow (accessibility audit).
AC-M07-11: 11-language smoke test: no truncated strings; ICU plural forms correct (i18n test).
AC-M07-12: availability >= 99.5% over rolling 30-day window (SLO burn-rate alert).
AC-M07-13: USSD path delivers polling station name within 2 SMS round-trips (USSD simulator test).

## Applicable edge cases

connectivity:
EC-CON-02: 3G SLAs met; CDN serves cached lookups; mobile-first page weight <= 100 KB.
EC-CON-05: flapping SMS gateway; OTP retry budget exhausted; email OTP fallback activates.
EC-CON-06: captive portal; cert pinning fails closed; "network blocked" UX; no token leaked.
EC-CON-08: DNS poisoned; DNSSEC enforced; hardcoded fallback CDN endpoints used.

time:
EC-TIM-04: DST/TZ: inspection window close computed in UTC; displayed in EAT; no off-by-one at boundary.
EC-TIM-08: petition window T+7 midnight; s.75(1) inclusive boundary; portal closes at exact UTC second.

identity:
EC-IDN-02: JWT expires mid-lookup; in-flight request completes; next request returns 401 and silent refresh.
EC-IDN-07: kiosk path uses device-bound FIDO2 credential; no SMS OTP for accredited operator.
EC-IDN-10: SpiceDB down; back-office objection admin fails closed; read-only portal degrades gracefully with cached register snapshot.
EC-IDN-11: realm key rotated; stale JWKS; refetch on kid miss; no 401 storm.

data:
EC-DAT-01: duplicate objection submission same key+body; cached response returned; no re-execution.
EC-DAT-02: same key different body; 422 idempotency-key-reuse returned.
EC-DAT-09: Unicode voter names; NFC normalisation; ICU collation; grapheme-safe display.
EC-DAT-13: soft-deleted voter; tombstone returned as "record not found"; no PII leaked.

security:
EC-SEC-05: injection on ID lookup and objection text inputs; parameterised queries only; ZAP and Semgrep clean.
EC-SEC-09: rate-limit evasion; key by subject, device, and /64 prefix; priority shed; IP block logged.
EC-SEC-10: voter enumeration timing oracle; constant-time response shape enforced; p95 delta < 5 ms verified.
EC-SEC-12: CDN cache poisoning; cache key includes signature version; client-side verification.
EC-SEC-15: CSRF + session binding + step-up re-auth required on third-party objection filing.
EC-SEC-16: secrets in logs refused; IP hash not raw IP; voter ID hash not raw NID.

human:
EC-HUM-06: screen readers; full task completion WCAG 2.2 AA on lookup and objection flows.
EC-HUM-07: KSL video explainer for objection filing; captions on all video content.
EC-HUM-08: 11 languages; no string truncation; ICU plural forms; font fallback for extended scripts.
EC-HUM-09: colour-blind; symbol + text + pattern on status indicators; ΔE registry.
EC-HUM-10: low-literacy USSD path; numeric flow <= 5 steps to polling station name.

scale:
EC-SCL-01: 5M lookups/hr at peak inspection day; >= 95% CDN edge hit ratio; origin p99 <= 500 ms.
EC-SCL-04: consumer lag > 60 s on audit event topic; backpressure applied; no OOM on audit writer.
EC-SCL-08: Valkey eviction loses objection idempotency key; PG-durable store used; Valkey is cache only.

legal:
EC-LEG-09: T-60 auto-referral race for objections; single referral issued; idempotent scheduler.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | rate-limiter, anti-enumeration shim, OTP validator, field masker, objection FSM | nextest | M07-UNIT- |
| property | timing oracle: response time constant regardless of voter existence | proptest | M07-PROP- |
| contract | BVR register read endpoint (provider M02) | Pact | M07-PACT- |
| integration | register-read with BVR double; SMS gateway double; audit log | nextest + testcontainers | M07-INT- |
| e2e | citizen self-lookup -> OTP -> redacted result; objection filing + tracking | Playwright | M07-E2E- |
| load | 100 000 concurrent lookups; IP-level rate-limit at threshold | k6 | M07-LOAD- |
| security | timing oracle; enumeration attempt; OWASP ZAP; Semgrep | ZAP + Semgrep | M07-SEC- |
| a11y | WCAG 2.2 AA; screen-reader; keyboard-only | axe + manual | M07-A11Y- |
| i18n | EN + SW; 11 locales | i18n suite | M07-I18N- |

## Runbooks and dashboards

- docs/runbooks/bvi-svc-degradation.md: extend with BVV-specific enumeration-attack response and kiosk-reset procedure.
- platform/observability/bvi-svc/grafana-bvv.json: lookup rate, objection queue depth, enumeration alert count.
- platform/observability/bvi-svc/slo-alerts.yaml: 99.5% burn rate; p99 lookup latency breach.

## Open questions

- Enumeration vs DPA tension: the per-lookup audit log (FR-M07-010) may conflict with ODPC guidance on retention of IP metadata; legal review needed before G3.
- sw/ spec references "Cloudflare" as CDN provider; ADR stack uses platform-provided CDN edge; text in sw/ should be corrected to remove the vendor name.
