# M37 voter-education-cms

| field | value |
|---|---|
| M-number | M37 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M37-voter-education-cms.md |

## Purpose

Manage, version-control, fact-check, translate, and distribute voter-facing educational materials across 11 Kenyan languages and four formats (video, radio/audio, print-PDF, WhatsApp-ready card); gate every asset behind a fact-check-partner review and IEBC Director approval before publication; distribute approved content to M13 Public Verifier App, M40 Media Portal, county-coordinator inboxes, and the WhatsApp campaign asset store; ensure no outdated or unreviewed content reaches voters.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| CMS Authoring and Approval UI (IEBC officers, reviewers) | Phoenix LiveView | services/training-svc/ex/cms_web/ |
| Director Approval Queue | Phoenix LiveView | services/training-svc/ex/cms_web/ |
| County Distribution Dashboard | Django | services/training-svc/django/county_admin/ |
| ODPC Analytics Audit Panel | Django | services/training-svc/django/county_admin/ |
| Voter Content Feed (M13 integration, public) | Astro | services/training-svc/astro/education-public-site/ |
| API | Rust (axum) | services/training-svc/crates/training-api/ |

Languages: EN + SW baseline; additional targets per item from Kikuyu, Luo, Kamba, Kalenjin, Luhya, Mijikenda, Meru, Embu, Somali, Turkana (11 languages total).

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ContentItem | id, title, language (BCP-47), format (ARTICLE/VIDEO/AUDIO/PRINT_PDF/WHATSAPP_CARD), topic_category, target_audience, election_cycle_tag, author_id, current_version_id, status (DRAFT/IN_FACT_CHECK/FACT_CHECK_PASSED/IN_APPROVAL/APPROVED/PUBLISHED/ARCHIVED/EXPIRED), approved_by, approved_at, expiry_date, created_at, updated_at | B | T1-consensus |
| ContentVersion | id, content_item_id, version_number, body_markdown, asset_file_id (blob reference), parent_version_id, created_by, created_at | C | T2 |
| FactCheckReview | id, content_item_id, version_id, reviewer_id, annotations (text/note/resolved), decision (APPROVED/RETURNED), completed_at | B | T1-consensus |
| DistributionEvent | id, content_item_id, channel (M13_APP/M40_MEDIA/COUNTY_EMAIL/WHATSAPP_STORE/CDN_PUSH), county_code, dispatched_at, quantity (for physical materials) | C | T2 |

## APIs

OpenAPI file: `packages/openapi-contracts/training/voter-education.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/cms/content | 2 | JWT+SpiceDB (IEBC_EDITOR) | yes | create content item |
| PUT /v1/cms/content/{id}/versions | 2 | JWT+SpiceDB (IEBC_EDITOR) | yes | save new version |
| POST /v1/cms/content/{id}/submit-review | 2 | JWT+SpiceDB (IEBC_EDITOR) | yes | submit for fact-check |
| PATCH /v1/cms/content/{id}/fact-check | 2 | JWT+SpiceDB (REVIEWER) | yes | submit fact-check decision + annotations |
| POST /v1/cms/content/{id}/approve | 2 | JWT+SpiceDB (IEBC_DIRECTOR) | yes | approve and trigger distribution |
| GET /v1/cms/content | 2 | JWT+SpiceDB (IEBC_EDITOR) | n/a | list content items (filterable) |
| GET /v1/cms/content/public | 1 | public | n/a | approved published items for M13 feed; 1000/min CDN-cached |
| GET /v1/cms/content/{id}/asset | 1 | CDN signed URL | n/a | stream or download asset via CDN |
| POST /v1/cms/distribution/{id}/physical | 2 | JWT+SpiceDB (COUNTY_COORD) | yes | record physical dispatch with quantity |
| GET /v1/cms/analytics | 2 | JWT+SpiceDB (IEBC_DIRECTOR) | n/a | aggregate content consumption report (no individual tracking) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.training.content_published.v1 | emits | eel.training.content.v1 | operational | yes | at-least-once idempotent |
| eel.training.content_expired.v1 | emits | eel.training.content.v1 | operational | yes | at-least-once idempotent |
| eel.training.content_archived.v1 | emits | eel.training.content.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M13 Public Verifier App | internal | integrations/verifier/ | yes | primary voter-facing content feed; CDN cache fallback |
| M40 Media Portal | internal | integrations/media-portal/ | yes | media partner distribution queue |
| M11 Public Portal | internal | integrations/portal/ | yes | supplementary publication of key approved materials |
| M28 AI Monitoring Stack | internal | integrations/ai-monitoring/ | yes | optional: AI-assisted plain-language check before fact-check submission |
| WhatsApp Business API | external | integrations/whatsapp/ | yes (stub) | campaign asset store push; 6-8 week lead time for API approval |
| CDN | external | integrations/cdn/ | yes | signed URL asset delivery; max-age 1 h; expiry triggers cache purge |
| Virus-scan service | internal | integrations/virus-scan/ | yes | upload screening before storage |
| MinIO | internal | integrations/minio/ | yes | encrypted asset object store |

## ML and AI

- Optional M28 E1-assisted plain-language check before fact-check submission; human IEBC officer reviews suggestion; not a gate.

## Dependencies

- Modules: M11, M13, M28, M40
- Platform: none
- Packages: eel-outbox-rs, eel-resilience-rs

## Feature checklist

1. FR-M37-001: structured article editor; mandatory fields: title, language, format, topic category, target audience, election-cycle tag, author
2. FR-M37-002: asset upload: video (MP4 <= 500 MB), audio (MP3 <= 100 MB), PDF (<= 20 MB), image card (PNG/JPG <= 5 MB); virus-scanned on upload; stored encrypted in MinIO
3. FR-M37-003: version control; every save creates immutable version; branch from any prior version; diff view; published version cannot be edited; new version required for corrections
4. FR-M37-004: fact-check workflow; assigned to fact-check partner reviewer from rotation list; 5 business-day review SLA; reviewer approves or returns with annotated notes; author addresses each note before resubmission
5. FR-M37-005: IEBC Director approval gate; after fact-check pass, enters Director approval queue; Director can approve, return, or escalate to Commissioner; approved content immutably marked with approver ID and timestamp
6. FR-M37-006: translation workflow; approved EN master triggers translation tasks per configured target languages; translator uploads translated text; IEBC Education Officer reviews and publishes localised version independently per language
7. FR-M37-007: 11 languages supported: EN + SW baseline; Kikuyu, Luo, Kamba, Kalenjin, Luhya, Mijikenda, Meru, Embu, Somali, Turkana
8. FR-M37-008: format variants per content item (video, audio, print, WhatsApp-card); all variants share same approval status; variant-specific approval not required separately
9. FR-M37-009: distribution channels: M13 content feed, M40 Media Portal, county-coordinator inboxes (email + portal notification), WhatsApp campaign asset store
10. FR-M37-010: content expiry; tag with expiry date; expired content auto-archived; no longer served to voters; retained for audit
11. FR-M37-011: aggregate consumption metrics (views, downloads) per content item; no individual tracking; county-level aggregation only; daily report to IEBC Voter Education Officer
12. FR-M37-012: search and discovery; topic, language, format; full-text search with language-aware stemming
13. FR-M37-013: physical distribution tracking; county coordinator records dispatch with quantity and date; system tracks coverage vs county population
14. Immediate retract-and-republish workflow for published content with factual error; M13 push notification of correction; version history preserved
15. Fact-check escalation to secondary reviewer at day 4 if primary unresponsive; IEBC Director can override with documented justification
16. Two-step translation quality check for minority languages: translator + community validator; quality scorecard per language
17. CDN max-age headers 1 hour; content expiry triggers CDN cache purge event; no stale expired content served

## Acceptance criteria

1. AC-M37-01: >= 95% fact-check reviews completed within 5 business days; review completion timestamp
2. AC-M37-02: >= 80% of core catalogue available in all 11 languages; language-coverage report
3. AC-M37-03: M13 content feed availability >= 99.9%; CDN uptime monitor
4. AC-M37-04: physical distribution coverage >= 90% of target counties by T-30; distribution dashboard
5. AC-M37-05: zero unreviewed content published; automated gate check on publish action
6. AC-M37-06: WCAG 2.2 AA pass on all voter-facing content pages; independent accessibility audit

## Applicable edge cases

**data**
- EC-DAT-13: voter-facing content published with factual error; retract-and-republish workflow; M13 push notification of correction; version history preserved for audit

**security**
- EC-SEC-03: reviewer attempts to publish directly; role gate denies; only IEBC_DIRECTOR can approve; reviewer scoped read+annotate only
- EC-SEC-07: external reviewer accesses unapproved content outside review queue; SpiceDB denies; only content in their assigned queue visible

**operations**
- EC-OPS-02: CDN origin failure; M13 falls back to cached content with staleness indicator
- EC-OPS-03: WhatsApp Business API approval delayed; SMS broadcast and M13 carry full load; WhatsApp is supplementary

**scale**
- EC-SCL-01: 1M content requests per day from M13 on election day; CDN edge absorbs; p99 <= 1 s
- EC-SCL-03: 500 active content items across 11 languages; up to 5,500 content-version records; T2 store scales horizontally

**human**
- EC-HUM-06: WCAG 2.2 AA on voter-facing content and CMS authoring UI
- EC-HUM-08: 11 languages; community validator for minority-language quality assurance

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | fact-check state machine, expiry auto-archive trigger, distribution-channel push logic, version immutability guard | nextest | M37-UNIT- |
| contract | M13 content feed, M40 media portal push, CDN signed URL | Pact | M37-PACT- |
| integration | content publish to M13 feed; CDN signed-URL generation; WhatsApp asset store push; physical dispatch recording | testcontainers | M37-INT- |
| e2e | draft video item, fact-check approval, director approval, verify M13 feed updated within 5 min | Playwright | M37-E2E- |
| security | reviewer attempts direct publish; external reviewer out-of-queue access; signed URL expiry | ZAP | M37-SEC- |
| a11y | WCAG 2.2 AA on M13 content feed and CMS UI; 3 languages | axe-core + screen reader | M37-A11Y- |
| load | 1M content requests/day against CDN; p99 <= 1 s | k6 | M37-LOAD- |
| chaos | CDN origin failure; M13 fallback to cached content with staleness indicator | Chaos Mesh | M37-CHAOS- |

Tags: `#[trace("FR-M37-005","AC-M37-05","EC-DAT-13")]`

## Runbooks and dashboards

- `docs/runbooks/training-svc-degradation.md` - extend with CMS CDN origin failure and distribution-channel failure paths
- `docs/runbooks/content-retract-and-republish.md` - new runbook; factual-error response; M13 push notification steps; CDN purge
- Grafana dashboard: `platform/observability/training-svc/voter-education-dashboard.json` (named `eel-iems-37`)
- SLO alerts: `platform/observability/training-svc/voter-education-slo-alerts.yaml` (alert ruleset `iems-37-alerts`; M13 feed 99.9%)

## Open questions

- WhatsApp Business API approval: 6-8 week lead time; IEBC must initiate application at T-120; fallback plan (SMS + M13) must be documented before go-live.
- CDN provider: Cloudflare is ADR-forbidden; use Akamai-class or Kenyan CDN (Kenya-peered nodes); confirm provider selection before architecture review.
- Community validators for minority languages: IEBC must contract per-language validators; contract terms and schedules required before translation pipeline activates.
- Fact-check partner rotation list: IEBC must identify and onboard external CSO fact-check partners before CMS goes live; partner accounts are external and scoped read+annotate only.
