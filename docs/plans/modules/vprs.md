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

Portal and PWA
1. FR-M04-001: responsive Astro web portal + PWA installable on Android and iOS; service-worker caches form for offline drafting.
2. FR-M04-002: biographic capture form covers all 22 BVR fields; biometric subset deferred to kit capture.
3. FR-M04-003: polling station selection: voter picks preferred station from geo-enabled list backed by M18 boundaries.
4. FR-M04-004: confirmation screen: IPRS pre-populated fields shown for applicant review before final submit.
5. FR-M04-005: application status page: citizen views progress, downloads QR code, and cancels application.

IPRS integration
6. FR-M04-006: IPRS identity verification: applicant enters national ID number; vprs-svc calls IPRS; biographic fields pre-populated on match.
7. FR-M04-007: IPRS circuit-breaker: fail-open on identity lookup (proceed; verify async); fail-closed on final submit until IPRS confirms.
8. FR-M04-008: IPRS duplicate result handling: if IPRS returns two matching persons, create AdjudicationCase; no auto-merge; block submission.

OTP and MFA
9. FR-M04-009: SMS OTP challenge sent to mobile number registered with applicant's national ID; delivery p95 <= 60 s.
10. FR-M04-010: email OTP fallback when SMS delivery fails; fallback offered after first SMS timeout.
11. FR-M04-011: OTP rate-limit: max 3 OTP requests per session; 15-minute lockout after 3 consecutive failures.

Consent and DPA
12. FR-M04-012: consent capture per Kenya DPA 2019: explicit tick-box for IEBC and IPRS processing; retention terms disclosed; right-to-withdraw explained.
13. FR-M04-013: consent withdrawal flow: citizen cancels application; deletion queued; ODPC audit log updated.

QR handoff token
14. FR-M04-014: QR / handoff token generated on successful submission; 30-day expiry; extendable to 90 days on citizen request.
15. FR-M04-015: handoff token single-use: consumed token returns 410 Gone; re-scan shows helpdesk instructions.
16. FR-M04-016: application auto-expires after 90 days with citizen notification via SMS and email before expiry.

KIEMS kit API
17. FR-M04-017: KIEMS kit retrieves pre-filled form via GET /vprs/v1/handoff/{token}; kit authenticates with SVID mTLS.
18. FR-M04-018: handoff endpoint response shape identical for valid, expired, and consumed tokens; timing delta p95 < 5 ms.

Anti-abuse
19. FR-M04-019: reCAPTCHA v3 on portal entry and form submission; audio CAPTCHA alternative for accessibility mode.
20. FR-M04-020: rate-limit: 60 req/min per IP, 10 req/min per session, 3 submission attempts per ID number per day.
21. FR-M04-021: anti-enumeration: ID lookup constant-time; response shape and timing identical for existing and non-existing IDs.
22. FR-M04-022: duplicate application prevention: second submission for same ID number returns 409 with reference to existing application.

Admin back-office
23. FR-M04-023: admin analytics dashboard: county-level conversion funnel (submitted, IPRS-verified, QR-issued, converted); no PII in results.
24. FR-M04-024: exception queue: admin resolves applications stuck in IPRS-pending for more than 24 hours.
25. FR-M04-025: ODPC compliance report: monthly data-processing activity report generated automatically and delivered to ODPC.
26. FR-M04-026: admin RBAC: Viewer, Analyst, Operator, Super-Admin roles; every admin action emitted to audit topic.

Accessibility and i18n
27. FR-M04-027: WCAG 2.1 AA on every screen; keyboard-only navigable; screen-reader compatible (NVDA, VoiceOver, TalkBack).
28. FR-M04-028: English + Kiswahili at launch; architecture extensible to 11 languages; no string truncation at 150% pseudo-locale.
29. FR-M04-029: colour-blind: all status indicators use symbol + text + pattern; ΔE >= 3.
30. FR-M04-030: USSD path for low-literacy / no-data applicants; numeric menu <= 5 steps; IVR alternative.

Compliance and security
31. FR-M04-031: zero PII in browser storage; all application state session-scoped; no localStorage or IndexedDB write with voter data.
32. FR-M04-032: security headers on every response: CSP, HSTS (includeSubDomains; preload), X-Frame-Options DENY, X-Content-Type-Options nosniff.
33. FR-M04-033: RBAC + full audit trail; Kenya DPA 2019 and ISO 27001:2022 compliance controls documented.
34. FR-M04-034: every mutation signed and emitted on eel.vprs.application.v1 topic; at-least-once idempotent delivery via outbox.
35. FR-M04-035: secrets: no PII, tokens, or ID numbers in logs or traces; Tier A/B telemetry fields rejected at collection layer.

Performance and availability
36. FR-M04-036: API p95 <= 200 ms; p99 <= 500 ms at 500 000 applications/day peak load.
37. FR-M04-037: availability cycle-critical 99.9% on 30-day rolling window; CDN offloads static assets; origin auto-scales.
38. FR-M04-038: source code + documentation + IPR transferred to IEBC at Operational Acceptance; handover certificate issued.

## Acceptance criteria

AC-M04-01: end-to-end apply-to-QR-to-KIEMS-conversion flow completes without errors under normal load (UAT).
AC-M04-02: 500 000 applications/day sustained; API p95 <= 200 ms; p99 <= 500 ms (load test, k6).
AC-M04-03: WCAG 2.1 AA independent audit passes on every screen; zero critical axe violations (accessibility audit).
AC-M04-04: IPRS circuit-breaker: fail-open on read; application proceeds; fail-closed on submit without IPRS confirmation (chaos exercise).
AC-M04-05: ODPC audit rated compliant; consent log and withdrawal test pass (ODPC review).
AC-M04-06: source code + docs + IPR transferred to IEBC at OAT; handover certificate signed (governance gate).
AC-M04-07: anti-enumeration timing oracle: p95 delta < 5 ms between existing and non-existing ID lookups (security harness).
AC-M04-08: handoff token single-use; consumed token returns 410 Gone; no second redemption possible (integration test).
AC-M04-09: SMS OTP delivery p95 <= 60 s; email fallback triggers on first SMS gateway timeout (integration test).
AC-M04-10: 11 languages render without truncation at 150% pseudo-locale; ICU plurals correct across locales (i18n test).
AC-M04-11: zero PII found in browser localStorage, sessionStorage, or IndexedDB after session close (security scan).
AC-M04-12: availability 99.9% measured over 30-day rolling window; SLO alert fires on breach (SLO measurement).
AC-M04-13: duplicate submission for same ID number returns 409 with reference to existing application (integration test).
AC-M04-14: audio CAPTCHA alternative completes end-to-end for screen-reader persona without visual challenge (a11y test).

## Applicable edge cases

connectivity
EC-CON-02: 3G (500 kbps, 300 ms RTT): portal p95 200 ms met via CDN edge; CBOR+zstd negotiated for API responses.
EC-CON-05: flapping link: PWA service-worker retries with backoff+jitter; idempotency key prevents duplicate submission.
EC-CON-06: captive portal: cert pinning in PWA fails closed; "network blocked" UX shown; no auth headers leak.
EC-CON-08: DNS poisoned/down: DNSSEC enforced; hardcoded fallback API endpoint available for OTP challenge flow.

data
EC-DAT-01: duplicate submission same key+body: cached 200 returned; no re-execution of IPRS lookup or OTP issue.
EC-DAT-02: same idempotency key, different body: 422 idempotency-key-reuse; applicant redirected to existing application.
EC-DAT-09: Unicode voter names: NFC normalisation on input; ICU collation in admin search; grapheme-safe display.
EC-DAT-13: soft-deleted application in admin search: tombstone honoured; row excluded from results; purge after 12 months.

security
EC-SEC-05: injection on every input: parameterised queries only; Semgrep compile-time; ZAP DAST scan; fuzz corpus.
EC-SEC-06: SSRF via redirect or callback URL fields: domain allowlist; no private ranges; DNS rebinding guard.
EC-SEC-09: rate-limit evasion via rotating IPs: key by subject + device + /64; reCAPTCHA on write path; priority shed.
EC-SEC-10: voter enumeration via ID lookup and handoff: constant-time shape; response caps; step-up auth on repeated misses.
EC-SEC-12: cache poisoning of application status snapshots: cache key includes signature version; client verifies before render.
EC-SEC-15: Astro session hijack: CSRF token + session binding enforced; re-auth required on step-up actions.
EC-SEC-16: secrets in logs/traces: telemetry refuses Tier A/B fields; log-scanner CI regex gate.

identity
EC-IDN-02: JWT expires mid-request: in-flight request completes; next request gets 401 + silent Keycloak refresh.
EC-IDN-07: SIM swap on citizen OTP: OTP device-bound for citizen path; new device requires re-verification.
EC-IDN-10: SpiceDB down: admin back-office fails closed on write; class-C reads served from cached policy decisions.
EC-IDN-11: realm key rotated, stale JWKS: vprs-svc refetches JWKS on kid miss; portal retries auth silently.

scale
EC-SCL-01: 5M visitors/hr at registration window peak: >= 95% requests served from CDN edge; origin p99 <= 500 ms.
EC-SCL-02: thundering herd on application status polling: SDK jitter; ETag 304 for unchanged status; rate-limit enforced.
EC-SCL-04: Kafka consumer lag > 60 s on eel.vprs.application.v1: SRE paged; backpressure applied; no OOM.

human
EC-HUM-06: screen readers: full WCAG 2.1 AA task completion verified with axe-core and manual NVDA/VoiceOver.
EC-HUM-07: deaf / KSL: KSL video explainer on consent and submission screens; captions present and verified.
EC-HUM-08: 11 languages, long strings: no truncation at 150% pseudo-locale; ICU plurals correct; font fallback stack.
EC-HUM-09: colour-blind: all status indicators symbol + text + pattern; ΔE >= 3 contrast threshold validated.
EC-HUM-10: low-literacy USSD path: numeric menu <= 5 steps for application status; IVR alternative for voice.

mobile (PWA)
EC-MOB-01: OS kills PWA mid-sync: service-worker resumes form state on restart; no duplicate submission triggered.
EC-MOB-03: 1 GB RAM device: PWA bundle <= 15 MB; p99 2 s on 3G; no OOM on low-end KES-10 000 handset.

time
EC-TIM-09: register freeze race at T-60/T-30: OPA hard-close at transaction time; no partial application accepted after freeze.

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
| a11y | WCAG 2.1 AA; keyboard-only; screen-reader (NVDA + VoiceOver) | axe-core + manual | M04-A11Y- |
| i18n | EN + SW; 11 locales; 150% pseudo | i18n test suite | M04-I18N- |

## Runbooks and dashboards

- docs/runbooks/vprs-svc-degradation.md (create): IPRS down procedure; fallback OTP; peak traffic shedding.
- platform/observability/vprs-svc/grafana-portal.json: application rate, conversion funnel, OTP delivery latency, IPRS circuit state.
- platform/observability/vprs-svc/slo-alerts.yaml: 99.9% burn rate; OTP p95 latency; IPRS circuit open.

## Open questions

- IPRS latency: IPRS p99 latency during national registration peak is unknown; circuit-breaker thresholds need empirical tuning at T-90.
- Enumeration abuse: the voter self-lookup surface (M07/BVV) creates a related enumeration risk; the two modules should share a common constant-time response shim.
