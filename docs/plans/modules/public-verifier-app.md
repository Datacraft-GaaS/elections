# M13 public-verifier-app

| field | value |
|---|---|
| M-number | M13 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.7 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M13-public-verifier-app.md |

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

1. FR-M13-001: ballot receipt QR scan: mobile app decodes QR code from printed ballot receipt and extracts encrypted ballot token.
2. FR-M13-002: manual code entry: citizen types 16-character ballot token directly when QR is unreadable or unavailable.
3. FR-M13-003: inclusion-proof verification: app fetches Merkle inclusion proof from transparency log and verifies ballot hash is committed.
4. FR-M13-004: Ed25519 signature verification: app verifies receipt signature against published election authority public key bundle.
5. FR-M13-005: chain-of-custody display: app shows per-ballot station timeline including receipt, seal, transmission, and count events.
6. FR-M13-006: transparency-log root download: citizen can download signed Merkle root snapshot for independent audit.
7. FR-M13-007: signed snapshot audit: app verifies downloaded root snapshot signature before displaying or exporting.
8. FR-M13-008: zero-knowledge voter privacy: no PII is returned or inferred from any verification query; only inclusion status returned.
9. FR-M13-009: web app ballot verification: Astro/React SSR surface with signed responses supports full ballot verification flow.
10. FR-M13-010: web app chain-of-custody view: timeline component renders custody events with station labels and timestamps.
11. FR-M13-011: web app transparency-log export: citizen downloads signed root JSON via browser; no server stores download intent.
12. FR-M13-012: Flutter mobile app ballot QR scan: camera integration with zxing-compatible decoder; fallback to gallery image.
13. FR-M13-013: Flutter offline Merkle verifier: bundled pure-Dart proof verifier works with no network after initial root sync.
14. FR-M13-014: Flutter offline civic content cache: polling-centre lookup, candidate lists, and how-to-verify guide cached for 7 days.
15. FR-M13-015: Flutter auto-sync on reconnect: app resumes proof sync and log root update when connectivity restores.
16. FR-M13-016: Flutter background root sync: app pulls latest signed root periodically over Wi-Fi to keep offline verifier current.
17. FR-M13-017: Flutter OTA update check: app checks for mandatory update at launch using Play/App Store metadata; blocks if critical.
18. FR-M13-018: Flutter battery-aware sync: background sync deferred when battery below 15% unless user initiates manually.
19. FR-M13-019: Flutter compressed offline pack: offline asset bundle gzip-compressed; fetched over GSM on first install.
20. FR-M13-020: Flutter low-storage graceful degradation: app sheds non-essential cached data when device storage below 50 MB.
21. FR-M13-021: Flutter roaming cost warning: app detects roaming SIM and warns before fetching large proof bundles.
22. FR-M13-022: anti-clone binary attestation: app checks Play Integrity / App Attest at launch; rejects if signature or store listing mismatches.
23. FR-M13-023: fake-app reporting relay: detected clone app signature reported to M16 fraud module.
24. FR-M13-024: USSD channel ballot token verify: citizen dials short code, enters token, receives inclusion status in <= 5 menu steps.
25. FR-M13-025: USSD results lookup: constituency and national summary retrievable via USSD within 182-char page constraint.
26. FR-M13-026: USSD stateless session: no server session state held between USSD requests; all context re-derived from input.
27. FR-M13-027: USSD low-literacy numeric menus: all USSD menus use numeric choices only; no alphabetic input required.
28. FR-M13-028: USSD latency SLA: p95 end-to-end USSD response under 5 seconds including gateway and backend round-trip.
29. FR-M13-029: USSD 182-char budget enforcement: all USSD response pages validated at build time to not exceed 182 characters.
30. FR-M13-030: WhatsApp Business API ballot verification: citizen sends ballot token to official WA number; bot returns inclusion status.
31. FR-M13-031: WhatsApp chain-of-custody summary: WA bot sends plain-text custody timeline; no attachments on base plan.
32. FR-M13-032: WhatsApp grievance filing: citizen describes incident in natural language; bot extracts structured fields and relays to M23.
33. FR-M13-033: WhatsApp quota-aware retry: WA API quota consumption tracked; bot queues and retries within daily limits.
34. FR-M13-034: WhatsApp USSD fallback: when WA quota exhausted, bot replies with USSD short code and guidance.
35. FR-M13-035: WhatsApp language detection: bot detects English vs Kiswahili from message content and responds in kind.
36. FR-M13-036: WhatsApp rate limiting: per-number message rate limited to prevent bot abuse; excess requests rejected with 429 hint.
37. FR-M13-037: public REST API ballot proof endpoint: GET /v1/verify/{token} returns inclusion proof JSON with Ed25519 signature.
38. FR-M13-038: public REST API chain-of-custody endpoint: GET /v1/custody/{token} returns ordered event list with station metadata.
39. FR-M13-039: public REST API transparency root endpoint: GET /v1/log/root returns latest signed Merkle root and timestamp.
40. FR-M13-040: public REST API signed response headers: all API responses include X-Signature header with Ed25519 over response body.
41. FR-M13-041: public REST API rate limiting: per-IP and per-token rate limits enforced; 429 returned with Retry-After header.
42. FR-M13-042: public REST API bot protection: proof-of-work or CAPTCHA challenge on unauthenticated bulk query patterns.
43. FR-M13-043: public REST API CORS policy: restrictive CORS with explicit allowed-origin list for browser consumers.
44. FR-M13-044: multilingual English support: all surfaces (web, Flutter, USSD menus, WA bot) serve English by default.
45. FR-M13-045: multilingual Kiswahili support: all surfaces serve Kiswahili; USSD menus truncated to 182-char per locale.
46. FR-M13-046: multilingual 9 additional languages: web and Flutter extensible to 11 total locales via i18n resource bundles.
47. FR-M13-047: WCAG 2.2 AA on web app: colour contrast, keyboard navigation, skip links, focus indicators, and ARIA roles.
48. FR-M13-048: WCAG 2.2 AA on PWA: same as web; service-worker offline page also accessible.
49. FR-M13-049: screen reader support on Flutter: Semantics widgets on all interactive elements; VoiceOver and TalkBack tested.
50. FR-M13-050: font scaling support: web and Flutter layouts reflow gracefully at 200% text size; no content truncation.
51. FR-M13-051: high-contrast mode: web app respects prefers-contrast media query; Flutter respects platform accessibility flags.
52. FR-M13-052: per-query audit log: channel, query type, result status, anonymised actor session hash, and timestamp logged to M15.
53. FR-M13-053: no PII in audit log: ballot token hashed before logging; IP addresses truncated to /24 for IPv4, /48 for IPv6.
54. FR-M13-054: grievance relay to M23: grievance payload forwarded within 30 seconds; relay acknowledgement stored.
55. FR-M13-055: grievance photo upload: citizen attaches photo evidence; stored in M03 object store with signed URL; no direct serving.
56. FR-M13-056: grievance audio upload: citizen records voice note on Flutter; same upload path as photo.
57. FR-M13-057: grievance content screening: grievance text forwarded to M17 for speech-integrity flag before relay to M23.
58. FR-M13-058: duplicate grievance deduplication: same ballot token + description hash within 10 minutes returns cached relay ack.
59. FR-M13-059: results relay from M11: signed constituency and national snapshots displayed with inclusion-proof link.
60. FR-M13-060: canary deployment support: app and API service deployable via canary with feature flags; gradual traffic shift.
61. FR-M13-061: blue-green zero-downtime deploy: verifier service deployable with no dropped requests during switchover.
62. FR-M13-062: SLA 99.9% during election cycle: uptime target measured over 7-day election window; automated burn-rate alert.
63. FR-M13-063: p95 200 ms web response SLA: CDN-cached proof responses served under 200 ms p95 from edge.
64. FR-M13-064: p95 500 ms mobile response SLA: non-cached proof fetch from origin under 500 ms p95 on 3G.
65. FR-M13-065: CDN edge caching for proof responses: proof responses cached at CDN edge with short TTL; invalidated on log root update.
66. FR-M13-066: 10 million concurrent users: load architecture supports 10M concurrent sessions across all surfaces combined.
67. FR-M13-067: queue transparency: live queue length at any polling station fetched from M19 and displayed in app.
68. FR-M13-068: polling centre lookup: citizen searches by name, GPS, or constituency; returns centre details and queue estimate.
69. FR-M13-069: civic how-to-verify guide: step-by-step in-app guide explaining ballot verification process; available offline.
70. FR-M13-070: candidate list display: official candidate list per constituency fetched from M11 and cached locally.
71. FR-M13-071: secret management: API keys, Ed25519 private keys, and WA tokens stored in vault; never in env files or logs.
72. FR-M13-072: secrets-in-logs prevention: structured log pipeline strips known secret patterns before writing to Loki.
73. FR-M13-073: dependency pinning: all eel- package versions pinned in lockfile; automated PR on upstream release.
74. FR-M13-074: public API versioning: /v1/ prefix; breaking changes increment to /v2/; both served during transition window.
75. FR-M13-075: rate-limit headers: X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset on all API responses.

## Acceptance criteria

AC-M13-01: ballot token inclusion proof verified by reference Dart client against live transparency log returns valid within p95 500 ms on 3G (integration test).
AC-M13-02: Ed25519 receipt signature verification passes for valid token and fails with explicit error for tampered or unknown token (unit + integration test).
AC-M13-03: USSD token verify completes in <= 5 menu steps with all responses under 182 characters per page (UAT + automated 182-char budget test).
AC-M13-04: WhatsApp quota exhausted scenario triggers USSD fallback message to citizen within one reply (chaos test).
AC-M13-05: grievance filed via WA or Flutter is relayed to M23 with full structured payload within 30 seconds (integration test with M23 double).
AC-M13-06: clone app with mismatched binary signature is blocked at launch and event reported to M16 (security test on Android + iOS builds).
AC-M13-07: WCAG 2.2 AA compliance confirmed on web app by independent axe-core scan with zero critical violations (a11y audit).
AC-M13-08: Flutter offline Merkle verifier returns correct inclusion result for 1,000 proof vectors without network (unit test, pre-bundled vectors).
AC-M13-09: 99.9% availability measured over simulated 7-day election-cycle load test; no more than 43 minutes downtime equivalent (load + chaos test).
AC-M13-10: p95 web proof response under 200 ms from CDN edge under 10M concurrent simulated users (k6 load test).
AC-M13-11: all API responses include valid X-Signature header verifiable with published election authority key (contract test).
AC-M13-12: no PII (voter name, ID number, address) appears in any audit log entry, API response, or error message for any surface (security audit + log scan).
AC-M13-13: Kiswahili locale renders correctly on all surfaces with no untranslated fallback strings and USSD pages within 182-char budget (i18n test suite).
AC-M13-14: canary deployment of new verifier build completes with zero dropped requests and error rate below 0.01% during traffic shift (staging deploy test).

## Applicable edge cases

EC-DAT-01: duplicate ballot token query within cache TTL returns cached proof response without re-fetching from transparency log.
EC-DAT-02: malformed or truncated ballot token input rejected with structured 400 error; no partial proof lookup attempted.
EC-SEC-05: SQL/NoSQL injection attempt in token field sanitised at input layer; parameterised queries throughout backend.
EC-SEC-16: Ed25519 private key never written to application logs; structured log pipeline strips key-shaped byte sequences.
EC-IDN-02: citizen with no registered device attempts verification via USSD and web; both paths work without device identity.
EC-IDN-10: ballot token recycled across election cycles distinguished by election-epoch prefix; old-cycle token returns explicit expired error.
EC-SCL-04: burst of 500K requests in 10 seconds absorbed by CDN edge cache; origin request rate stays within capacity envelope.
EC-MOB-01: low-end KES-10k Android device with 1 GB RAM loads verifier app within 4 seconds; lazy loading prevents OOM.
EC-MOB-02: gzip-compressed offline pack fetched over GPRS/GSM connection completes within 3 minutes; progress indicator shown.
EC-MOB-03: OTA update pushed to Flutter app over GSM without Wi-Fi; user prompted; update applied on next launch if space allows.
EC-MOB-04: device storage drops below 50 MB mid-sync; app sheds oldest cached civic content and retries with reduced pack.
EC-MOB-05: battery drops to 14% during background root sync; sync deferred and user notified on next foreground entry.
EC-MOB-06: SIM switches to international roaming; app detects and warns before downloading proof bundle over roaming data.
EC-MOB-07: Play Store / App Store listing pulled mid-election; app falls back to web surface URL displayed to citizen.
EC-CON-01: citizen on no-data plan uses USSD path exclusively; all verification and grievance functions reachable via USSD.
EC-CON-02: 3G congestion; CDN-cached proof response under 500 ms p95; USSD offered as fallback link in web UI.
EC-CON-03: 2G-only handset; USSD path selected automatically; no multimedia content pushed.
EC-CON-04: packet loss above 10%; Flutter app retries proof fetch with exponential backoff; USSD fallback suggested after 3 failures.
EC-CON-05: intermittent connectivity mid-grievance submission; app buffers payload locally and retries on reconnect.
EC-CON-06: VPN or proxy detected; API still responds; rate-limit keyed on /48 subnet to avoid over-blocking.
EC-CON-07: satellite link with 600 ms RTT; timeout tuned per surface; offline civic cache prevents re-fetch of static content.
EC-CON-08: ISP-level DNS hijacking; app uses DNS-over-HTTPS pinned resolver; USSD unaffected by DNS.
EC-IDN-01: first-time citizen with no prior digital footprint verifies ballot via USSD with token only; no account required.
EC-IDN-03: voter who cast ballot at extension table not in original station list; chain-of-custody still shows extension event.
EC-IDN-04: custom ROM triggers false fail on Play Integrity; app offers USSD and web paths with clear message.
EC-IDN-05: ballot token issued to voter in diaspora; proof fetch works cross-border; chain of custody shows overseas station.
EC-IDN-06: duplicate token collision (negligible probability); both holders receive inclusion proof; audit flag raised to M15.
EC-IDN-07: token entered with extra whitespace or lowercase hex; app normalises before lookup; no error for clean input.
EC-IDN-08: citizen under duress sends covert signal via app; escalation relayed to M23 without visible confirmation in UI.
EC-IDN-09: ballot token from spoilt ballot included in log; inclusion proof returns with spoilt-ballot flag; UI explains meaning.
EC-IDN-11: minor voter (legally eligible in jurisdiction) verifies ballot; no age gate on verifier; privacy rules same as adult.
EC-SCL-01: election-day traffic spike 10x baseline; auto-scaling triggers within 60 seconds; no p99 breach.
EC-SCL-02: CDN cache miss storm after root invalidation; origin protected by request coalescing and stampede lock.
EC-SCL-03: database connection pool exhausted under load; circuit breaker trips; CDN serves stale proof cache.
EC-SCL-05: 50 million proof requests over 48-hour election window; log storage pre-allocated; no disk-full failure.
EC-SCL-06: WA webhook throughput spikes; queue depth monitored; WA quota burn-rate alert at 80%.
EC-SCL-07: 10M concurrent users p99 < 500 ms; CDN edge absorbs 95% of proof traffic.
EC-SCL-08: USSD gateway saturated; additional gateway instances spun up via infrastructure automation.
EC-HUM-01: citizen with visual impairment uses screen reader on web; all interactive elements have accessible labels.
EC-HUM-02: citizen with motor impairment uses keyboard-only navigation on web; focus order logical and visible.
EC-HUM-03: citizen with low literacy selects USSD numeric menu; all choices single-digit; no reading required beyond number.
EC-HUM-04: elderly citizen unfamiliar with QR uses manual code entry path; field labelled clearly; error message actionable.
EC-HUM-05: 11 locales loaded; RTL locale (if added) layout mirrors correctly; no overlap or truncation.
EC-HUM-06: VoiceOver on iOS and TalkBack on Android read chain-of-custody timeline in logical sequence.
EC-HUM-07: USSD menus translated to Kiswahili; all pages within 182-char budget in SW locale.
EC-HUM-08: WA bot switches language mid-conversation when citizen replies in alternate language.
EC-HUM-09: citizen with colour blindness; no information conveyed by colour alone; icons and labels accompany all status indicators.
EC-HUM-10: low-literacy USSD path; token entry split into two 8-digit halves across two prompts to reduce error rate.
EC-HUM-11: citizen types grievance text in local language not yet in i18n bundle; content stored as-is; M17 handles classification.
EC-HUM-12: large text mode (200% scale) on Flutter; layout reflows; no text clipped or overlapping.
EC-SEC-01: HTTPS enforced on all web and API endpoints; HSTS header set; no plain HTTP fallback.
EC-SEC-02: Content Security Policy blocks inline scripts on web app; nonce-based for SSR-injected scripts.
EC-SEC-09: token verification endpoint rate-limited per IP; constant-time response prevents timing oracle.
EC-SEC-10: enumeration via sequential token guessing detected by anomaly threshold; IP blocked after 50 attempts per minute.
EC-SEC-14: clone app detection via binary attestation; mismatched app blocked; event sent to M16.
EC-SEC-15: CSRF protection on web grievance form; double-submit cookie pattern.
EC-SEC-17: API key for WA integration rotated on schedule; old key invalidated within 60 seconds of rotation.
EC-LEG-03: right-to-erasure request on grievance PII actioned within 72 hours; audit trail of erasure retained without PII.
EC-LEG-08: data residency requirement; all proof and custody data stored in jurisdiction-local region; no cross-border replication.
EC-TIM-07: proof response timestamp drift > 5 minutes from citizen device clock; app warns about clock skew; proof still displayed.

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
