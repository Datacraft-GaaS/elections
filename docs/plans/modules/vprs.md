# M04 vprs

| field | value |
|---|---|
| M-number | M04 |
| Home service | vprs-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M04-vprs.md |

## Purpose

Let a Kenyan citizen pre-fill their voter registration form online, verify identity against IPRS, and receive a QR handoff token that lets a KIEMS enrolment clerk complete biometric capture in minutes, reducing polling-centre queuing and boosting registration coverage.

## Surfaces and languages

| surface | technology | path under services/vprs-svc/ |
|---|---|---|
| Public web portal + PWA | Astro on packages/eel-web-astro | astro/vprs-portal/ |
| Application API | Rust (axum + utoipa) | crates/vprs-api/ |
| Domain logic | Rust | crates/vprs-domain/ |
| Admin back-office | Django on packages/eel-web-py | django/vprs-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| VPRSApplication | app_id, id_number_hash, biographic_fields (22-subset), status, qr_token_hash, expires_at | B | T2 |
| AuditEvent | event_id, actor, action, ts_hlc, ip_hash | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/vprs/portal.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /vprs/v1/applications | 2 live-per-user | JWT | yes | Create pre-registration; IPRS check on submit |
| GET /vprs/v1/applications/{id}/status | 2 live-per-user | JWT | n/a | Citizen status check |
| DELETE /vprs/v1/applications/{id} | 2 live-per-user | JWT | yes | Citizen cancellation |
| GET /vprs/v1/handoff/{token} | 3 accredited | SVID mTLS | n/a | Kit retrieves pre-filled form |
| POST /vprs/v1/otp/request | 2 live-per-user | public | yes | SMS OTP challenge |
| POST /vprs/v1/otp/verify | 2 live-per-user | public | yes | OTP verification |
| GET /vprs/v1/admin/analytics | 3 accredited | JWT+SpiceDB | n/a | Admin analytics |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.vprs.application_submitted.v1 | emits | eel.vprs.application.v1 | operational | yes | at-least-once idempotent |
| eel.vprs.application_cancelled.v1 | emits | eel.vprs.application.v1 | operational | yes | at-least-once idempotent |
| eel.vprs.handoff_redeemed.v1 | emits | eel.vprs.application.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.enrolment_accepted.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| IPRS | I01 | integrations/iprs/ | yes | fail-open on read (allow application, verify async); fail-closed on submit |
| SMS gateway | I08 | integrations/sms-gw/ | yes | OTP retry; email OTP fallback |
| BVR back-end (M02) | internal | integrations/bvr-backend/ | yes | conversion event queued on reconnect |
| Transparency log (P05) | internal | integrations/tlog/ | yes | anchoring queued |

## ML and AI

none

## Dependencies

- Modules: M01 (handoff token consumer on kit), M02 (conversion event target)
- Platform: P04 (IAM, OTP), P05 (audit)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, eel-web-astro, eel-web-py

## Feature checklist

1. FR-M04-001: responsive Astro web portal + PWA (Android + iOS accessible).
2. FR-M04-002: biographic capture form with 22 BVR fields (biometric subset deferred to kit).
3. FR-M04-003: IPRS identity verification on ID number; pre-populate biographic fields.
4. FR-M04-004: SMS OTP challenge to mobile registered against applicant ID.
5. FR-M04-005: explicit consent per Kenya DPA 2019 with right-to-withdraw disclosed.
6. FR-M04-006: QR / handoff token generation; 30-day expiry; extendable to 90 days.
7. FR-M04-007: KIEMS kit API at /vprs/v1/handoff/{token} for pre-populated form retrieval.
8. FR-M04-008: application status portal; citizen can check progress and cancel.
9. FR-M04-009: admin back-office: analytics by county, conversion rate, exception queues.
10. FR-M04-010: reCAPTCHA on entry; rate-limit per IP and per ID; no register enumeration.
11. FR-M04-011: WCAG 2.2 AA on every screen; screen-reader compatible; keyboard-only navigable.
12. FR-M04-012: English + Kiswahili at launch; extensible to 11 languages.
13. FR-M04-013: RBAC + full audit trail; Kenya DPA 2019; ISO 27001:2022 compliance.
14. FR-M04-014: 500 000 applications/day sustainable at peak.
15. FR-M04-015: every mutation signed + audit-logged.
16. FR-M04-016: source code + documentation transferred to IEBC at OAT.
17. Page load p99 <= 2 s at national peak.
18. OTP delivery <= 60 s p95.
19. Zero PII in browser storage; session-scoped only.
20. IPRS circuit-breaker: fail-open on read; fail-closed on submission until verified.
21. Handoff token is single-use; consumed token returns 410 Gone.
22. Application auto-expires after 90 days with citizen notification.
23. Anti-enumeration: lookup by ID returns same response time regardless of existence.
24. Email OTP fallback when SMS delivery fails.
25. Admin analytics show county-level conversion funnel without PII.

## Acceptance criteria

AC-M04-01: end-to-end apply-to-QR-to-KIEMS-conversion under normal load (UAT).
AC-M04-02: 500 000 applications/day sustained (load test).
AC-M04-03: WCAG 2.2 AA independent audit pass.
AC-M04-04: IPRS circuit-breaker: fail-open on read; fail-closed on submit (chaos exercise).
AC-M04-05: ODPC audit rated compliant (ODPC review).
AC-M04-06: source code + docs + IPR transferred to IEBC at OAT (handover certificate).
AC-M04-07: anti-enumeration timing oracle p95 delta < 5 ms (security test).
AC-M04-08: handoff token single-use; consumed token returns 410 (integration test).

## Applicable edge cases

**connectivity**
EC-CON-02: 3G SLAs; CBOR+zstd negotiated for mobile portal.
EC-CON-08: DNS poisoned; DNSSEC; hardcoded fallback.

**identity**
EC-IDN-02: JWT expires mid-request; in-flight completes; next 401 + silent refresh.
EC-IDN-04: attestation fails (custom ROM); public USSD/SMS path offered.
EC-IDN-07: no SMS OTP for accredited kit path; device-bound token.
EC-IDN-11: realm key rotated; stale JWKS; refetch on kid miss.

**data**
EC-DAT-01: duplicate application same key+body; cached response.
EC-DAT-09: Unicode names; NFC normalisation.

**security**
EC-SEC-05: injection on every input; parameterised only; ZAP scan.
EC-SEC-06: SSRF via URL fields; allowlist; no private ranges.
EC-SEC-09: rate-limit evasion; key by subject+device+/64; priority shed.
EC-SEC-10: voter enumeration; constant-time shape; caps; step-up.
EC-SEC-15: LiveView session hijack (Astro: CSRF + session binding + re-auth on step-up).
EC-SEC-16: secrets in logs; telemetry refuses Tier A/B.

**human**
EC-HUM-06: screen readers; full task AA.
EC-HUM-08: 11 languages; no truncation.
EC-HUM-10: low-literacy USSD path; numeric <= 5 steps.

**legal**
EC-TIM-09: register freeze race; OPA hard-close; all-or-nothing.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | IPRS circuit-breaker, token expiry logic, consent capture, OTP rate-limiter | nextest | M04-UNIT- |
| property | anti-enumeration timing; token uniqueness; rate-limit correctness | proptest | M04-PROP- |
| contract | IPRS adapter (consumer); M01 handoff endpoint (provider); M02 conversion event | Pact | M04-PACT- |
| integration | IPRS double; SMS gateway double; BVR conversion flow | nextest + testcontainers | M04-INT- |
| e2e | citizen apply -> OTP -> QR -> kit handoff -> enrolment | Playwright (web) + patrol (kit) | M04-E2E- |
| fuzz | form inputs; OTP payload; URL fields (SSRF) | cargo-fuzz + Nuclei | M04-FUZZ- |
| load | 500 000 applications/day; OTP delivery latency | k6 | M04-LOAD- |
| chaos | IPRS blackhole; SMS gateway down; primary fail | Chaos Mesh | M04-CHAOS- |
| security | OWASP ZAP; Semgrep; timing-oracle test; WCAG audit | ZAP + axe | M04-SEC- |
| a11y | WCAG 2.2 AA; keyboard-only; screen-reader (NVDA + VoiceOver) | axe-core + manual | M04-A11Y- |
| i18n | EN + SW; 11 locales; 150% pseudo | i18n test suite | M04-I18N- |

## Runbooks and dashboards

- docs/runbooks/vprs-svc-degradation.md (create): IPRS down procedure; fallback OTP; peak traffic shedding.
- platform/observability/vprs-svc/grafana-portal.json: application rate, conversion funnel, OTP delivery latency, IPRS circuit state.
- platform/observability/vprs-svc/slo-alerts.yaml: 99.9% burn rate; OTP p95 latency; IPRS circuit open.

## Open questions

- IPRS latency: IPRS p99 latency during national registration peak is unknown; circuit-breaker thresholds need empirical tuning at T-90.
- Enumeration abuse: the voter self-lookup surface (M07/BVV) creates a related enumeration risk; the two modules should share a common constant-time response shim.
