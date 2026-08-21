# M07 bvv

| field | value |
|---|---|
| M-number | M07 |
| Home service | bvi-svc (ADR-0020) |
| Owning work package | WP-3.7 (docs/plans/04-wave-3-critical-path.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M07-bvv.md |

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
| GET /bvv/v1/objections | 3 accredited | JWT+SpiceDB | n/a | IEBC back-office objection queue |
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

1. FR-M07-001: public self-lookup by ID number + OTP to registered mobile.
2. FR-M07-002: IEBC kiosk self-service with QR-scan or manual ID entry.
3. FR-M07-003: return name, DOB (year only), polling station, ward, constituency, county, status.
4. FR-M07-004: mask sensitive fields; biometrics never displayed; full DOB masked; mobile masked.
5. FR-M07-005: file objection on own record (rectification, transfer, removal).
6. FR-M07-006: file objection on third-party record with justification + evidence upload.
7. FR-M07-007: objection tracking: citizen sees status of filed objection.
8. FR-M07-008: IEBC back-office objection queue with adjudication workflow and response templates.
9. FR-M07-009: anti-abuse: rate limit per IP + per session; CAPTCHA; alerts on enumeration patterns.
10. FR-M07-010: per-lookup audit entry: actor, target voter hash, timestamp, IP hash.
11. FR-M07-011: multilingual: English + Kiswahili; extensible.
12. FR-M07-012: WCAG 2.2 AA + screen-reader support.
13. FR-M07-013: Kenya DPA compliance: consent, purpose disclosure, retention policy shown.
14. Lookup p99 <= 2 s; 100 000 concurrent users during peak inspection days.
15. Per-session cap of 20 lookups/day enforced.
16. Every lookup permanently logged; log not erasable even on SAR.
17. Enumeration pattern alert fires at configured threshold; IP blocked; IEBC alerted.
18. Kiosk mode locks to single-session after idle timeout; no PII cached in browser.
19. OTP delivery <= 60 s p95; email OTP fallback if SMS fails.
20. Objection SLA: first response to objector <= 48 h; resolution <= 30 days.

## Acceptance criteria

AC-M07-01: 100 000 concurrent lookups sustained without p99 degradation (load test).
AC-M07-02: bulk-enumeration attempt blocked at configured threshold; IP blocked; alert fired (red-team).
AC-M07-03: ODPC audit rated compliant (ODPC review).
AC-M07-04: objection workflow closes cases within SLA (UAT).
AC-M07-05: per-lookup audit entry present for every request (audit trace).
AC-M07-06: no biometric or full DOB exposed in any response (automated + manual scan).
AC-M07-07: anti-enumeration timing oracle p95 delta < 5 ms (security test).

## Applicable edge cases

**connectivity**
EC-CON-02: 3G SLAs; CDN cached; mobile-first.

**identity**
EC-IDN-02: JWT expires mid-request; in-flight completes; 401 + refresh.
EC-IDN-07: OTP is device-bound for accredited kiosk path.
EC-IDN-11: stale JWKS; refetch on kid miss.

**data**
EC-DAT-09: Unicode voter names; NFC; grapheme-safe.
EC-DAT-13: soft-deleted voter in lookup; tombstone honoured.

**security**
EC-SEC-05: injection on ID lookup input; parameterised.
EC-SEC-09: rate-limit evasion; key by subject+device+/64; priority shed.
EC-SEC-10: voter enumeration; constant-time shape; caps; step-up.
EC-SEC-12: cache poisoning; key includes sig version.
EC-SEC-15: CSRF + session binding + re-auth on step-up.
EC-SEC-16: secrets in logs refused.

**human**
EC-HUM-06: screen readers; full task AA.
EC-HUM-08: 11 languages; no truncation.
EC-HUM-10: low-literacy; simple ID-entry flow.

**legal**
EC-TIM-08: petition window T+7 midnight; s.75(1) inclusive boundary.

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
