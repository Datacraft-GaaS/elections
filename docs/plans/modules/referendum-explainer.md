# M68 referendum-explainer

| field | value |
|---|---|
| M-number | M68 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-5.7 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M68-referendum-explainer.md |

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

1. FR-M68-001: Create explainer for each referendum question from M49.
2. FR-M68-002: Author plain-language summary in editorial CMS.
3. FR-M68-003: Machine-assisted NLLB translation for all 11 languages; human review gate before publish.
4. FR-M68-004: Generate TTS audio for each language version.
5. FR-M68-005: Attach KSL video for Deaf accessibility.
6. FR-M68-006: Add fact-check annotation per claim with verdict and source.
7. FR-M68-007: Require all 11 language translations complete before publish.
8. FR-M68-008: Publish explainer; content served via CDN edge.
9. FR-M68-009: WCAG 2.2 AAA compliance on all public surfaces.
10. FR-M68-010: Keyboard-only navigation complete on explainer portal.
11. FR-M68-011: Screen reader full task completion on every explainer flow.
12. FR-M68-012: Colour contrast ratio AAA (7:1) throughout portal.
13. FR-M68-013: KSL video captions and audio description track present.
14. FR-M68-014: Explainer content updated when referendum question amended in M49.
15. FR-M68-015: Public portal lists questions with status (draft/published).
16. FR-M68-016: Admin CMS shows translation progress per language.
17. FR-M68-017: Fact-check annotations public; source links verifiable.
18. FR-M68-018: Audit trail of all content edits and translations.
19. FR-M68-019: TTS audio regenerated automatically on text change.
20. FR-M68-020: Explainer available on M29 mobile app for offline reading.
21. FR-M68-021: Portal p95 200 ms / p99 500 ms at routine load.

## Acceptance criteria

1. AC-M68-01: WCAG 2.2 AAA: axe zero violations and manual screen-reader full-task script signed; M68-specific AAA criteria documented.
2. AC-M68-02: All 11 language translations required before publish; API returns 422 if any missing.
3. AC-M68-03: KSL video and captions present for every published question; asset-presence test.
4. AC-M68-04: TTS audio generated within 5 min of text publish; integration test.
5. AC-M68-05: Explainer updated within 60 s of referendum question amendment; e2e test.
6. AC-M68-06: Public portal p95 200 ms at routine 99.5% SLA.
7. AC-M68-07: 11 locale snapshot tests pass; pseudo-locale 150% no truncation.

## Applicable edge cases

**Human:**
- EC-HUM-06: screen readers; full task completion; AA + AAA for M68; axe + manual scripts.
- EC-HUM-07: Deaf / KSL video; KSL for every explainer; captions.
- EC-HUM-08: 11 languages; long strings; no truncation; font fallback.
- EC-HUM-09: colour-blind; ΔE for chart colours; AAA contrast ratio (7:1).

**Scale:**
- EC-SCL-01: 5M visitors/hr on explainer portal; 95% edge; CDN.
- EC-SCL-02: thundering herd on STH; ETag 304; jitter.

**Security:**
- EC-SEC-12: cache poisoning of explainer snapshots; key includes signature version.
- EC-SEC-08: XXE in translation import; DTD off.

**ML:**
- EC-ML-01: NLLB model drift; version pinned; human review mandatory.
- EC-ML-05: PII in translation prompts; PIIStrippingEvent before NLLB call.

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
