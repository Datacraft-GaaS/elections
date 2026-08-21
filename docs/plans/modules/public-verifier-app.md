# M13 public-verifier-app

| field | value |
|---|---|
| M-number | M13 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.7 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M13-public-verifier-app.md |

## Purpose

Multi-channel citizen verification app: lets any Kenyan verify their participation token, look up published results, file grievances, and access civic education across Flutter mobile, Astro PWA, USSD, WhatsApp, and an API surface consumed by observers and parties, targeting 10M concurrent users during result announcement.

## Surfaces and languages

| surface | technology | path under services/field-mobile-svc/ |
|---|---|---|
| Flutter citizen app (Android + iOS) | Flutter on packages/kit-sdk-flutter | apps/citizen-verify/ |
| Progressive web app | Astro on packages/eel-web-astro | astro/verifier-pwa/ |
| USSD gateway | Rust USSD session handler | crates/verifier-ussd/ |
| WhatsApp bot | Rust (axum + WA Cloud API I11) | crates/verifier-wa/ |
| Public verifier API | Rust (axum + utoipa) | crates/verifier-api/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| VerificationQuery | query_id, channel, actor_session_hash, query_type, result_status, ts | C | T1-audit |
| ParticipationTokenQuery | query_id, token_sha256, inclusion_proof_ref, verified_at | D | T2 |
| GrievanceRef | grievance_id, channel, submitted_at, ack_ref | C | T2 (relay to M23) |
| CivicContentCache | item_id, lang, body_html, hash, cached_at | D | Valkey |

## APIs

OpenAPI file: packages/openapi-contracts/verifier/public.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /verify/v1/results | 1 signed-static | public | n/a | Relay from M11; CDN-cached |
| GET /verify/v1/results/{scope}/{code} | 1 signed-static | public | n/a | Constituency/regional results relay |
| POST /verify/v1/token | 2 live-per-user | public | yes | Verify participation token hash; inclusion proof |
| GET /verify/v1/token/{sha}/proof | 2 live-per-user | public | n/a | Return inclusion proof JSON |
| POST /verify/v1/grievances | 2 live-per-user | public | yes | File grievance (relay to M23) |
| GET /verify/v1/civic/{item_id} | 1 signed-static | public | n/a | Civic content from M11 CMS |
| GET /verify/v1/queue/{station} | 2 live-per-user | public | n/a | Queue snapshot from M19 |
| GET /verify/v1/anomalies | 2 live-per-user | public | n/a | Public anomaly/fraud flags from M26 |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.verifier.token_queried.v1 | emits | eel.verifier.audit.v1 | analytics | yes | at-least-once idempotent |
| eel.verifier.grievance_relayed.v1 | emits | eel.verifier.audit.v1 | operational | yes | at-least-once idempotent |
| eel.rts.snapshot_published.v1 | consumes | eel.rts.snapshot.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| Public portal M11 | internal | integrations/public-portal/ | yes | CDN relay; stale cache clearly labelled |
| RTS back-end M10 | internal | integrations/rts-backend/ | yes | token proof lookup; circuit breaker |
| Reproducibility bundle M14 | internal | integrations/reproducibility/ | yes | token inclusion proof |
| Grievance channels M23 | internal | integrations/grievance/ | yes | relay queued; retry |
| Queue transparency M19 | internal | integrations/queue/ | yes | queue snapshot relay |
| Speech integrity M17 | internal | integrations/speech/ | yes | content flag relay |
| ECC M16 | internal | integrations/ecc/ | yes | anomaly flag relay |
| WhatsApp Cloud API | I11 | integrations/whatsapp/ | yes | WA quota; 6-8 wk approval lead time |
| USSD gateway | I08 | integrations/sms-gw/ | yes | session-based; 182-char limit |
| Valkey (platform) | internal | platform/valkey/ | yes | civic content cache |

## ML and AI

none (speech integrity screening happens in M17; this module forwards grievance text to M17 for content classification)

## Dependencies

- Modules: M10 (results + proof), M11 (public portal relay), M14 (token proof), M16 (ECC anomalies), M17 (content screening), M19 (queue data), M23 (grievance relay)
- Platform: P04 (IAM, optional JWT for authenticated paths), Valkey
- Packages: eel-crypto-rs, eel-web-astro, kit-sdk-flutter

## Feature checklist

1. FR-M13-001: participation token verification: citizen enters token hash; app returns inclusion proof + station.
2. FR-M13-002: results relay from M11: signed constituency/national snapshots with inclusion proof link.
3. FR-M13-003: grievance filing: citizen describes incident; photo/audio upload; relay to M23.
4. FR-M13-004: civic content: how-to-verify guide, polling centre lookup, candidate list.
5. FR-M13-005: queue transparency: live queue length at any polling station from M19.
6. FR-M13-006: USSD path: token verify + results lookup in <= 5 steps; 182-char constraint.
7. FR-M13-007: WhatsApp bot: token verify + results + grievance in natural language (EN + SW).
8. FR-M13-008: Flutter app: offline civic content cache; auto-sync on reconnect.
9. FR-M13-009: anti-clone: app pinned to official store listing; binary attestation.
10. FR-M13-010: WCAG 2.2 AA on web and PWA surfaces; screen-reader on Flutter.
11. FR-M13-011: multilingual: English + Kiswahili at launch; extensible to 11 languages.
12. FR-M13-012: 10M concurrent users; p99 < 500 ms on CDN-cached paths.
13. FR-M13-013: content screening relay: grievance text forwarded to M17 for speech integrity flag.
14. FR-M13-014: WA rate limiting: quota-aware retry; fallback to USSD if quota exhausted.
15. FR-M13-015: per-query audit log: channel, query type, result status, actor session hash.
16. 10M concurrent; p95 <= 200 ms; p99 <= 500 ms.
17. USSD session latency p95 <= 4 s; WA response p95 <= 10 s.
18. Anti-clone binary attestation checked at app launch; fake apps reported to M16.
19. Flutter offline mode: civic content 7-day cache; graceful stale on results.

## Acceptance criteria

AC-M13-01: 10M concurrent; p99 < 500 ms on CDN-cached results (load test).
AC-M13-02: participation token inclusion proof verified by reference client (integration test).
AC-M13-03: USSD token verify completes in <= 5 steps (UAT).
AC-M13-04: WA quota exhausted; fallback to USSD initiated (chaos test).
AC-M13-05: grievance relayed to M23 within 30 s (integration test).
AC-M13-06: clone app blocked by attestation (security test).
AC-M13-07: WCAG 2.2 AA on PWA (independent audit).
AC-M13-08: 99.9% availability during election cycle (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-02: 3G SLAs; CDN-cached; USSD fallback.
EC-CON-03: 2G only; USSD path; no multimedia.
EC-CON-07: satellite; timeout tuned; offline civic cache.

**identity**
EC-IDN-04: attestation false-fail on custom ROM; USSD/web path offered.
EC-IDN-08: duress signal via app; covert escalation to M23.

**data**
EC-DAT-01: duplicate grievance same key; cached relay ack.
EC-DAT-09: Unicode input (names, grievance text); NFC normalisation.

**security**
EC-SEC-09: rate-limit on token verify; key by IP+/64.
EC-SEC-10: enumeration via token verify; constant-time; cap.
EC-SEC-14: clone app detection; attestation; binary integrity.
EC-SEC-15: CSRF + session binding on web paths.
EC-SEC-16: secrets in logs refused.

**scale**
EC-SCL-07: 10M concurrent; p99 < 500 ms; CDN edge sheds load.
EC-SCL-09: WA quota exhausted; failover to USSD.

**human**
EC-HUM-06: screen readers; WCAG 2.2 AA.
EC-HUM-08: 11 languages; USSD 182-char.
EC-HUM-10: low-literacy; USSD numeric <= 5 steps.

**mobile**
EC-MOB-01: KES-10k Android; low RAM; lazy loading.
EC-MOB-02: store approval required; binary attestation pinned.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | token proof verify, USSD session FSM, WA bot intent parser, grievance relay, quota-aware WA retry | nextest + Flutter test | M13-UNIT- |
| property | token verify always returns proof or error; USSD session never hangs | proptest | M13-PROP- |
| contract | M10 proof endpoint (consumer); M11 results relay (consumer); M23 grievance (provider) | Pact | M13-PACT- |
| integration | M10 double; M11 double; WA double; USSD double; grievance relay | nextest + testcontainers | M13-INT- |
| e2e | citizen: token verify on Flutter; results on PWA; grievance on WA; USSD token verify | patrol + Playwright | M13-E2E- |
| fuzz | token hash; grievance text; USSD input; WA webhook payload | cargo-fuzz | M13-FUZZ- |
| load | 10M concurrent; CDN edge; USSD gateway; WA webhook rate | k6 | M13-LOAD- |
| chaos | M11 disconnect; WA quota hit; USSD gateway down; M23 relay fail | Chaos Mesh | M13-CHAOS- |
| security | clone detection; attestation bypass; OWASP ZAP; Semgrep | ZAP + Semgrep + OWASP Mobile | M13-SEC- |
| a11y | WCAG 2.2 AA on PWA; VoiceOver on Flutter | axe-core + manual | M13-A11Y- |
| i18n | EN + SW; 11 locales; USSD 182-char budget test | i18n suite | M13-I18N- |

## Runbooks and dashboards

- docs/runbooks/verifier-svc-degradation.md (create): WA quota exhausted; USSD gateway down; M11 disconnect.
- platform/observability/field-mobile-svc/grafana-verifier.json: query rate by channel, token verify success rate, grievance relay lag, WA quota remaining.
- platform/observability/field-mobile-svc/slo-alerts.yaml: 99.9% burn rate; p99 > 500 ms; WA quota < 20%.

## Open questions

- WhatsApp approval: WA Cloud API business verification takes 6-8 weeks; application must start at T-12 months; any delay pushes WA channel out of scope.
- Clone app reporting: mechanism for citizens to report clone apps to M16 needs UX design; current plan is in-app report button.
