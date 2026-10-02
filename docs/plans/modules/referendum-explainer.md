# M68 referendum-explainer

| field | value |
|---|---|
| M-number | M68 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-5.7 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M68-referendum-explainer.md |

## Purpose

Publishes authoritative, accessible explainers for each referendum question: plain-language text, KSL video, TTS audio, fact-check annotations, and 11-language translations, targeting WCAG 2.2 AAA compliance as the primary accessibility standard.

## Surfaces and languages

| surface | technology | path under services/portal-svc/ |
|---|---|---|
| Public referendum explainer portal | Astro | astro/explainer/ |
| Explainer CMS (editorial) | Elixir (Phoenix LiveView) | ex/portal_live/ (explainer editorial) |
| TTS batch worker | Python | py/tts/ |
| Internal API | Rust (axum) | crates/gazette/ (shared explainer endpoints) |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ReferendumQuestion | id, referendum_id (FK M49), question_number, official_text, plain_language_summary, status (draft/fact-checked/published), published_at | C | T1-consensus |
| ExplainerContent | id, question_id, language_code, text_html, audio_url, video_ksl_url, tts_generated_at, translator_id, reviewed_at | C | T1-consensus |
| FactCheckAnnotation | id, question_id, claim_text, verdict (accurate/inaccurate/context), source_url, annotated_by, annotated_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/explainer/referendums/{referendum_id}/questions | 1 signed-static | public | n/a | list questions with explainer status |
| GET /v1/explainer/questions/{id} | 1 signed-static | public | n/a | full explainer per question and language |
| GET /v1/explainer/questions/{id}/audio | 1 signed-static | public | n/a | TTS audio file |
| GET /v1/explainer/questions/{id}/ksl | 1 signed-static | public | n/a | KSL video link |
| POST /v1/explainer/questions | 3 accredited | JWT+SpiceDB | yes | create question explainer |
| PUT /v1/explainer/questions/{id}/content/{lang} | 3 accredited | JWT+SpiceDB | yes | upsert translation |
| POST /v1/explainer/questions/{id}/fact-check | 3 accredited | JWT+SpiceDB | yes | add fact-check annotation |
| PUT /v1/explainer/questions/{id}/publish | 3 accredited | JWT+SpiceDB | yes | publish explainer |
| POST /v1/explainer/questions/{id}/tts | internal gRPC | SVID mTLS | yes | trigger TTS generation |

OpenAPI: packages/openapi-contracts/portal/referendum-explainer.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.portal.explainer_published.v1 | emits | eel.portal.explainer.v1 | operational | no | at-least-once idempotent |
| eel.portal.tts_generated.v1 | emits | eel.portal.explainer.v1 | analytics | no | at-least-once idempotent |
| eel.governance.referendum_activated.v1 | consumes | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M49 referendum-mechanics | internal | crates/gazette/ | yes | advisory; question list source |
| M17 speech-integrity | internal | crates/gazette/ | yes | advisory; fact-check support |
| M11 public-portal | internal | crates/gazette/ | yes | advisory; portal cross-link |
| M29 mobile-citizen-verify | internal | crates/gazette/ | yes | advisory; explainer surfaced in app |
| M23 grievance-channels | internal | crates/gazette/ | yes | advisory; citizen queries on referendum |
| NLLB (AI translation via ai-platform) | litellm | integrations/litellm/ | yes | circuit breaker; human translator alert |

## ML and AI

- NLLB via LiteLLM on ai-platform for machine-assisted translation of all 11 languages; human review mandatory before publish.
- TTS (Python py/tts/) for audio explainers; voice model version pinned.

## Dependencies

- Modules: M49 (referendum questions), M17 (fact-check), M11 (portal), M29 (mobile)
- Platform: P04 (Kafka outbox), MinIO (audio/video assets)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-astro

## Feature checklist

Question registry:

1. FR-M68-001: Register each referendum ballot question as a named entity: question_id (UUID7), official text verbatim from Gazette, question type (constitutional_amendment / repeal / approval), ballot-paper position, and constitutional provisions affected. (FR-68.01)
2. FR-M68-002: Seed question registry from the official Gazette notice; any post-seed change to official question text requires Commissioner sign-off and an audit log entry. (FR-68.01)
3. FR-M68-003: On receipt of eel.governance.referendum_activated.v1 from M49, create a draft ExplainerContent record per ballot question for every required language.

Explainer content authoring:

4. FR-M68-004: Each explainer carries mandatory structured sections: plain-language summary (<= 150 words), Yes-vote legal effect, No-vote legal effect, constitutional context (Art. citation + <= 100 words), up to 10 FAQ pairs, fact-check status, and numbered source citations. (FR-68.02)
5. FR-M68-005: the Commission Content Editor creates the explainer draft in the CMS; all mandatory sections must be populated before the draft can advance to Legal Review. (FR-68.03)
6. FR-M68-006: Legal Reviewer confirms constitutional citation accuracy; approval recorded with reviewer UUID and timestamp; regression to Draft permitted with mandatory written reason. (FR-68.03)
7. FR-M68-007: Commissioner signs off each explainer before publication; sign-off requires 2FA; digital signature appended to the version record and stored in T1-audit. (FR-68.03)
8. FR-M68-008: Editorial state machine enforces sequence: Draft, Legal Review, Fact-Check Review, Commissioner Approval, Published; no stage may be skipped; reversion requires mandatory reason text. (FR-68.03)
9. FR-M68-009: CMS shows a completion dashboard per question per language; any language below 100% at T-60 triggers an alert to the Commission Comms.

Fact-check partner integration:

10. FR-M68-010: Registered fact-check partners (PesaCheck, Code for Africa, others registered by the Commission) access explainer content via authenticated API; partners annotate specific claims with verdict (accurate / inaccurate / unverifiable) and evidence URL. (FR-68.04)
11. FR-M68-011: Any inaccurate verdict from a fact-check partner blocks state advance to Commissioner Approval until the editor resolves the annotation. (FR-68.04)
12. FR-M68-012: Editor resolution options: update content triggering automatic re-check, or override with a written determination; resolution recorded with editor UUID and narrative. (FR-68.04)
13. FR-M68-013: Fact-check annotations publicly visible on the explainer page; each annotation shows partner name, verdict badge, and evidence URL. (FR-68.04)

Multi-language pipeline:

14. FR-M68-014: English master content approved before any translation workflow begins; EN is the single source of truth for all derived content. (FR-68.05)
15. FR-M68-015: Swahili translation is mandatory; explainer publish endpoint returns 422 if any required language version is absent. (FR-68.05)
16. FR-M68-016: Machine-assisted translation via NLLB (LiteLLM on ai-platform) for all 11 languages; human reviewer sign-off mandatory before each language version is published. (FR-68.05)
17. FR-M68-017: Translator assigned per language; bilingual community reviewer co-signs before the language version advances to Published state. (FR-68.05)
18. FR-M68-018: Each language version carries its own publication state; publishing EN does not auto-publish any other language. (FR-68.05)
19. FR-M68-019: PII stripped from all content passed to NLLB via PIIStrippingEvent before the external LiteLLM call; enforced by Semgrep rule and regression corpus. (EC-ML-05)

Multi-format output:

20. FR-M68-020: From approved EN content, auto-generate: web page (responsive HTML, WCAG 2.2 AAA), PDF (A4, branded, print-ready), TTS audio (py/tts worker via M17 Speech Integrity), plain-text SMS (<= 160 chars summary per question), easy-read version (simplified language, Makaton-compatible symbols), and KSL video embed. (FR-68.06)
21. FR-M68-021: TTS audio regenerated automatically within 5 minutes of any text change to a published explainer; regeneration tracked via eel.portal.tts_generated.v1 event. (FR-68.06)
22. FR-M68-022: SMS format constrained to "Q[n]: [short text]. YES=[effect]. NO=[effect]." at <= 160 chars; CMS previews truncation before approval. (FR-68.06)
23. FR-M68-023: Easy-read version generated as a separate HTML page and PDF; simplified sentence structure applied; Makaton-compatible symbol set embedded. (FR-68.06)
24. FR-M68-024: KSL sign-language video commissioned at T-90 and uploaded to MinIO; linked from ExplainerContent.video_ksl_url; captions and audio description track required; publish gate blocked until both are present. (FR-68.06 / FR-68.08)
25. FR-M68-025: Radio bulletin plain-text export per question (max 300 words) available via authenticated GET endpoint for KBC and other broadcasters. (FR-68.07)

Distribution channels:

26. FR-M68-026: M11 Public Portal embeds explainer widgets via GET /v1/explainer/questions/{id}; embed includes all format tabs (web, audio, PDF, KSL). (FR-68.07)
27. FR-M68-027: M49 Referendum Mechanics retrieves contextual explainer link at the ballot interaction point; link served from signed-static CDN URL. (FR-68.07)
28. FR-M68-028: M29 Mobile Citizen Verify in-app tab shows explainer per question; content pre-cached for offline reading. (FR-68.07)
29. FR-M68-029: SMS push distributed via M23 to opted-in subscribers on first publication and on material corrections; push triggered within 60 minutes of new version. (FR-68.07)
30. FR-M68-030: WhatsApp Business API delivers explainer on demand when voters query the Commission official bot with a question number. (FR-68.07)

Accessibility:

31. FR-M68-031: All web explainer pages achieve WCAG 2.2 AAA; colour contrast ratio >= 7:1 throughout; no automatic audio; extended audio description available for KSL videos. (FR-68.08 / NFR-68.01)
32. FR-M68-032: Keyboard-only navigation completes all core tasks (read explainer, switch language, play audio, download PDF) without mouse. (FR-M68-010)
33. FR-M68-033: Screen-reader full task completion verified on NVDA (Windows), VoiceOver (macOS/iOS), and TalkBack (Android) before each explainer version is published. (FR-68.08)
34. FR-M68-034: All KSL videos carry closed captions; audio description track provided for visual-only content within videos. (FR-68.06 / FR-68.08)
35. FR-M68-035: Dark-mode and high-contrast mode support; contrast ratio >= 7:1 maintained in both light and dark themes; verified by automated contrast checker.
36. FR-M68-036: USSD numbered-menu path exposes question list in <= 5 steps; IVR audio alternative available for feature-phone users. (EC-HUM-10)

Versioning and correction log:

37. FR-M68-037: Each published explainer version is immutable; corrections create a new version with incremented version number; prior versions accessible via ?version= query parameter. (FR-68.09)
38. FR-M68-038: Public change log per question shows version number, change description, date, and approver; rendered on the explainer page. (FR-68.09)
39. FR-M68-039: Material corrections trigger push notifications via all distribution channels that delivered the original version; notification dispatched within 60 minutes of new version publication. (FR-68.09)

Analytics and reach:

40. FR-M68-040: Collect anonymous page-level analytics: page views per question, audio plays, PDF downloads, language distribution, device type; no voter-identifying tracking, no cookies required. (FR-68.10)
41. FR-M68-041: Analytics endpoint (GET /v1/explainer/public/analytics) exposes aggregate reach data per question and language; used by the Commission to prioritise translation effort. (FR-68.10)
42. FR-M68-042: Post-referendum transparency report includes reach analytics exported from M68; published on M11 Public Portal. (FR-68.10)

Content integrity:

43. FR-M68-043: All published explainer versions signed by HSM via P06; signature verified by consumer SDKs before rendering. (NFR-68.01)
44. FR-M68-044: Cache keys include HSM signature version; stale signed copy fails client-side verification and triggers CDN cache bust. (EC-SEC-12)
45. FR-M68-045: No cookies set for anonymous explainer reads; CSP strict-mode with no inline scripts; HTTPS only; no external analytics beacons. (NFR-68.04)

Performance and availability:

46. FR-M68-046: Explainer portal p99 <= 2 s on simulated 3G (500 kbps / 300 ms RTT); images optimised; TTS audio streamed; PDF generated server-side and cached. (NFR-68.03)
47. FR-M68-047: Public explainer content served with 99.9% availability from publication date to T+30; CDN origin failover to read replica on primary degradation. (NFR-68.02)
48. FR-M68-048: Portal handles 5 million concurrent page views at referendum vote-day peak; >= 95% served from CDN edge; origin p99 <= 500 ms for cache misses. (EC-SCL-01)

Observability:

49. FR-M68-049: All editorial state transitions emitted to P05 with eel_question_id, eel_editor_id, eel_language, and eel_version fields; transitions queryable by IAB auditors. (NFR-M68-MONITORING)
50. FR-M68-050: OpenTelemetry traces exported to central collector; Grafana dashboard (eel-M68) and Alertmanager ruleset (iems-M68-alerts) present before OAT gate. (NFR-M68-MONITORING)
51. FR-M68-051: NLLB translation model version pinned in config; drift alert fires if deployed model version differs from pinned value; human reviewer assignment triggered automatically on drift detection.

## Acceptance criteria

1. AC-M68-01: All published explainer web pages achieve WCAG 2.2 AAA; automated axe-core scan returns zero critical or serious violations; manual screen-reader walkthroughs on NVDA, VoiceOver, and TalkBack each complete all core tasks without assistive-technology failure; audit report signed by designated accessibility tester.
2. AC-M68-02: Colour contrast ratio >= 7:1 verified programmatically for every colour token in the portal design-system via automated contrast checker in CI; zero failures before OAT gate.
3. AC-M68-03: All 11 language translations required before publish; the publish endpoint returns 422 with eel_error_code: translations_incomplete if any required language version is absent or unapproved.
4. AC-M68-04: KSL video and captions present for every published question; asset-presence integration test confirms video_ksl_url and captions track both non-null before the publish gate passes.
5. AC-M68-05: TTS audio generated within 5 minutes of text publication; measured by integration test tracking time between explainer published_at timestamp and eel.portal.tts_generated.v1 event arrival.
6. AC-M68-06: Explainer portal CDN edge hit ratio >= 95% under 5M requests/hr k6 load test; origin p99 <= 500 ms for cache misses; origin p99 <= 2 s under simulated 3G throttle (500 kbps, 300 ms RTT).
7. AC-M68-07: Public explainer API (GET /v1/explainer/questions/{id}) p99 <= 200 ms at routine steady-state load; service availability class: routine 99.5%.
8. AC-M68-08: Inaccurate fact-check annotation blocks state advance; integration test confirms publish attempt returns 422 when any annotation carries verdict: inaccurate and resolved: false.
9. AC-M68-09: Commissioner sign-off requires 2FA; sign-off attempt without completed MFA returns 403 with eel_error_code: mfa_required; verified by integration test.
10. AC-M68-10: All 11 locale snapshot tests pass with no string truncation; pseudo-locale at 150% text expansion renders without horizontal overflow at viewport widths >= 320 px.
11. AC-M68-11: PII stripped from all NLLB translation prompts; log-scanner fixture and regression corpus both return zero PII hits; Semgrep rule enforces PIIStrippingEvent call before any LiteLLM call.
12. AC-M68-12: Material correction on a published explainer triggers push notification to all opted-in SMS subscribers via M23 within 60 minutes of new version publication; verified by integration test with M23 double.
13. AC-M68-13: Secrets (JWTs, HSM key handles, NLLB API tokens) produce zero hits in log-scanner fixture (Tier A/B patterns); CI gate blocks merge on any hit (EC-SEC-16).
14. AC-M68-14: CDN cache poisoning of explainer snapshot detected; client SDK signature verification fails on tampered cached copy and triggers CDN cache bust; verified by Varnish injection test.
15. AC-M68-15: Anonymous analytics endpoint returns aggregate counts only; no voter-identifying fields present in any response; confirmed by ZAP DAST scan and schema-level enforcement in the Pydantic model.
16. AC-M68-16: SpiceDB unavailable during editorial write returns 503 problem+json; public CDN-served explainer read remains available via cached edge responses; verified by PodChaos test.
17. AC-M68-17: RTO for the explainer portal origin <= 1 hour; RPO <= 5 minutes; documented PITR drill completed before OAT gate.

## Applicable edge cases

EC-DAT-01: duplicate explainer draft creation with same question_id and language returns cached 201 with no duplicate state-machine instantiation.
EC-DAT-02: idempotency key reused with a different explainer body returns 422 eel_error_code: idempotency_key_reuse, preventing silent content substitution.
EC-DAT-06: ExplainerContent schema v1 (no audio_url field) and v2 coexist during migration; both accepted by the API; dual-emit ensures M29 and M11 consumers receive correct shape.
EC-DAT-09: Somali, Arabic, and Dholuo Unicode text in explainer body is NFC-normalised; no grapheme truncation in SMS (160-char limit) or PDF; verified by 11-locale snapshot tests.
EC-DAT-13: superseded explainer versions not shown in the default public response; tombstone honoured; prior versions accessible via ?version= query param; purge job does not remove audit entries.
EC-SEC-05: injection attempted via question text, source citation URLs, and fact-check annotation claim text; all inputs parameterised; ZAP DAST and Semgrep confirm zero raw interpolation.
EC-SEC-08: NLLB translation import or content import containing XML with DOCTYPE or ENTITY declaration (XXE); parser configured DTD disabled; billion-laughs fuzz case returns 400.
EC-SEC-09: rate-limit evasion on the public explainer API via rotating IPs; rate-limit key is subject plus device plus /64 subnet; priority shed applied; k6 rotating-IP test confirms.
EC-SEC-12: CDN cache poisoning of explainer snapshot; cache key includes HSM signature version; client SDK verifies signature before rendering; stale tampered copy returns verification failure and triggers CDN cache bust.
EC-SEC-16: NLLB API token, editor JWT, and HSM key handle must not appear in OpenTelemetry traces or application logs; log-scanner fixture in CI enforces zero Tier A/B matches.
EC-IDN-02: JWT expires mid-editorial review or mid-fact-check annotation submit; in-flight request completes on existing token; next request receives 401 and portal silently refreshes.
EC-IDN-09: dual-control Commissioner sign-off; if the second approver's account is revoked between first and second signature, the pending approval is invalidated and a new approver must be assigned.
EC-IDN-10: SpiceDB unavailable during editorial write; write fails closed with 503 problem+json; public CDN-served explainer read remains available via cached edge responses.
EC-HUM-06: screen-reader users (NVDA, VoiceOver, TalkBack) complete all core explainer tasks (read, language-switch, audio play, PDF download) without assistive-technology failure; WCAG 2.2 AAA manual audit script signed before OAT gate.
EC-HUM-07: Deaf users access KSL video for every published question; captions and audio description tracks present; asset-presence test enforced at publish gate; UAT conducted with Deaf panel before T-30.
EC-HUM-08: all 11 language locales active; long strings in Somali and Arabic do not overflow SMS boundaries or PDF certificate margins; ICU plurals applied; font fallback stacks verified across all locales.
EC-HUM-09: colour-blind simulation (deuteranopia, protanopia, tritanopia) applied to all portal colour tokens; AAA contrast ratio (7:1) maintained under all simulations; axe-core colour-contrast rule enforced in CI.
EC-HUM-10: low-literacy voter accesses referendum information via USSD in <= 5 numeric steps; IVR audio alternative available; USSD simulator walk included in M68-A11Y- test suite.
EC-SCL-01: 5 million visitors per hour at referendum announcement; >= 95% of requests served from CDN edge; origin p99 <= 500 ms for cache misses; verified by k6 plus Gatling 5-region load test.
EC-SCL-02: thundering herd on explainer content hash endpoint after mass SMS push; SDK jitter applied; ETag 304 responses absorb repeat fetches; rate-limit caps origin exposure.
EC-SCL-04: consumer lag on eel.portal.explainer.v1 topic exceeds 60 s; Alertmanager pages on-call; backpressure applied; no event loss; verified by slow-consumer integration test.
EC-ML-01: NLLB model drift detected; version-pinned model mismatches deployed model; drift alert fires; human-review assignment ratio rises until new model card is reviewed and approved.
EC-ML-05: PII present in explainer text passed to NLLB; PIIStrippingEvent fires before external call; regression corpus and Semgrep rule both confirm zero PII in any NLLB prompt.
EC-OPS-08: MinIO bucket quota reached during KSL video upload; service returns 507 with eel_error_code: storage_quota_exceeded; alert fires; no partial video written; publish gate blocked.
EC-CON-02: editor on 3G (500 kbps, 300 ms RTT) saves a large explainer draft; CBOR+zstd negotiated; save completes within the module SLA; editorial CMS handles degraded connectivity without data loss.
EC-TIM-04: DST transition on diaspora editors' devices; all timestamps stored in UTC; editorial deadline display converted to editor's gazetted time zone; no off-by-one on T-60 target calculations.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Explainer FSM, translation completeness gate, TTS trigger | nextest | M68-UNIT- |
| property | Translation completeness invariant; audio regeneration | proptest | M68-PROP- |
| contract | M49 referendum; NLLB/LiteLLM double | Pact | M68-PACT- |
| integration | Full explainer lifecycle; TTS; KSL asset | nextest + testcontainers | M68-INT- |
| e2e | Public explainer portal; CMS authoring; fact-check | Playwright | M68-E2E- |
| load | 5M/hr CDN-served portal | k6 + Gatling | M68-LOAD- |
| a11y | axe WCAG AAA; manual screen-reader script; KSL presence | axe-core + manual | M68-A11Y- |
| i18n | 11 locales + 150% pseudo; no truncation | custom | M68-I18N- |
| security | ZAP; cache-poisoning; log scanner | ZAP + Semgrep | M68-SEC- |

Tags: `#[trace("FR-M68-009","AC-M68-01","EC-HUM-06")]`

## Runbooks and dashboards

- docs/runbooks/portal-svc-degradation.md (extend with explainer section)
- docs/runbooks/tts-generation-failure.md (new)
- platform/observability/portal-svc/explainer-dashboard.json
- platform/observability/portal-svc/slo-alerts.yaml

## Open questions

- WCAG AAA manual script sign-off: who is the designated screen-reader tester and what is the audit cadence before election day?
- NLLB translation quality for minority Kenyan languages (Dholuo, Gikuyu, Somali): human reviewer capacity planning required.
