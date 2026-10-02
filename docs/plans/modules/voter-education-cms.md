# M37 voter-education-cms

| field | value |
|---|---|
| M-number | M37 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M37-voter-education-cms.md |

## Purpose

Manage, version-control, fact-check, translate, and distribute voter-facing educational materials across 11 Kenyan languages and four formats (video, radio/audio, print-PDF, WhatsApp-ready card); gate every asset behind a fact-check-partner review and the Commission Director approval before publication; distribute approved content to M13 Public Verifier App, M40 Media Portal, county-coordinator inboxes, and the WhatsApp campaign asset store; ensure no outdated or unreviewed content reaches voters.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| CMS Authoring and Approval UI (the Commission officers, reviewers) | Phoenix LiveView | services/training-svc/ex/cms_web/ |
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
| POST /v1/cms/content | 2 | JWT+SpiceDB (COMMISSION_EDITOR) | yes | create content item |
| PUT /v1/cms/content/{id}/versions | 2 | JWT+SpiceDB (COMMISSION_EDITOR) | yes | save new version |
| POST /v1/cms/content/{id}/submit-review | 2 | JWT+SpiceDB (COMMISSION_EDITOR) | yes | submit for fact-check |
| PATCH /v1/cms/content/{id}/fact-check | 2 | JWT+SpiceDB (REVIEWER) | yes | submit fact-check decision + annotations |
| POST /v1/cms/content/{id}/approve | 2 | JWT+SpiceDB (COMMISSION_DIRECTOR) | yes | approve and trigger distribution |
| GET /v1/cms/content | 2 | JWT+SpiceDB (COMMISSION_EDITOR) | n/a | list content items (filterable) |
| GET /v1/cms/content/public | 1 | public | n/a | approved published items for M13 feed; 1000/min CDN-cached |
| GET /v1/cms/content/{id}/asset | 1 | CDN signed URL | n/a | stream or download asset via CDN |
| POST /v1/cms/distribution/{id}/physical | 2 | JWT+SpiceDB (COUNTY_COORD) | yes | record physical dispatch with quantity |
| GET /v1/cms/analytics | 2 | JWT+SpiceDB (COMMISSION_DIRECTOR) | n/a | aggregate content consumption report (no individual tracking) |

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

- Optional M28 E1-assisted plain-language check before fact-check submission; huma Commission officer reviews suggestion; not a gate.

## Dependencies

- Modules: M11, M13, M28, M40
- Platform: none
- Packages: eel-outbox-rs, eel-resilience-rs

## Feature checklist

Authoring:
1. FR-M37-001: structured article editor; mandatory fields: title, language, format, topic category, target audience, election-cycle tag, author
2. FR-M37-002: asset upload: video (MP4 <= 500 MB), audio (MP3 <= 100 MB), PDF (<= 20 MB), image card (PNG/JPG <= 5 MB); virus-scanned on upload; stored encrypted in MinIO
3. FR-M37-003: version control; every save creates immutable version; branch from any prior version; diff view; published version cannot be edited; new version required for corrections
4. FR-M37-018: virus scan on every uploaded asset before storage; infected uploads rejected with error code; alert raised; no infected content stored in MinIO
5. FR-M37-021: content item status machine: DRAFT, IN_FACT_CHECK, FACT_CHECK_PASSED, IN_APPROVAL, APPROVED, PUBLISHED, ARCHIVED, EXPIRED; no backward transition without logged reason
6. FR-M37-022: content item locked for editing once submitted for fact-check; unlock requires withdrawal with logged reason and return to DRAFT
7. FR-M37-028: topic categories enumerated: REGISTRATION, HOW_TO_VOTE, VOTER_RIGHTS, CANDIDATE_CONDUCT, OBSERVER_RIGHTS, DISPUTE_RESOLUTION, POLLING_STATION, GENERAL
8. FR-M37-029: election-cycle tagging; content tagged to election cycle identifier; filtering and reporting by cycle; archived content queryable by cycle

Fact-check workflow:
9. FR-M37-004: fact-check workflow; assigned to fact-check partner reviewer from rotation list; 5 business-day review SLA; reviewer approves or returns with annotated notes; author addresses each note before resubmission
10. FR-M37-015: fact-check escalation to secondary reviewer at business day 4 if primary unresponsive; the Commission Director can override with documented justification
11. FR-M37-023: fact-check partner accounts scoped to read and annotate only; cannot publish, approve, or access content outside their assigned review queue; SpiceDB enforced

Approval:
12. FR-M37-005: the Commission Director approval gate; after fact-check pass, enters Director approval queue; Director can approve, return, or escalate to Commissioner; approved content immutably marked with approver ID and timestamp
13. FR-M37-027: M28 optional plain-language check before fact-check submission; AI suggestion presented to the Commission officer; human review required; not a gate

Translation:
14. FR-M37-006: translation workflow; approved EN master triggers translation tasks per configured target languages; translator uploads translated text; the Commission Education Officer reviews and publishes localised version independently per language
15. FR-M37-007: 11 languages supported: EN + SW baseline; Kikuyu, Luo, Kamba, Kalenjin, Luhya, Mijikenda, Meru, Embu, Somali, Turkana
16. FR-M37-016: two-step translation quality check for minority languages: translator uploads, community validator reviews; quality scorecard per language stored on version record

Formats and distribution:
17. FR-M37-008: format variants per content item (video, audio, print, WhatsApp-card); all variants share same approval status; variant-specific approval not required separately
18. FR-M37-009: distribution channels: M13 content feed, M40 Media Portal, county-coordinator inboxes (email + portal notification), WhatsApp campaign asset store; push on PUBLISHED event
19. FR-M37-020: adaptive video bitrate (360p/480p/720p) for low-bandwidth environments; offline download option for M13 app; bandwidth detection applied
20. FR-M37-026: CDN signed URL for asset delivery; URL TTL configurable (default 1 h); expired signed URLs return 403; no direct MinIO access exposed to public

Expiry and retraction:
21. FR-M37-010: content expiry; tag with expiry date; expired content auto-archived; no longer served to voters; retained for audit; expiry triggers eel.training.content_expired.v1 event
22. FR-M37-014: immediate retract-and-republish workflow for published content with factual error; M13 push notification of correction; version history preserved; retraction anchored in audit log
23. FR-M37-017: CDN max-age headers 1 h; content expiry triggers CDN cache purge event within 5 minutes; no stale expired content served after purge

Search and discovery:
24. FR-M37-012: search and discovery; topic, language, format; full-text search with language-aware stemming on M13 voter content feed

Analytics and reporting:
25. FR-M37-011: aggregate consumption metrics (views, downloads) per content item; no individual tracking; county-level aggregation only; daily report to the Commission Voter Education Officer
26. FR-M37-025: analytics DPIA compliance: no cookies beyond session on voter-facing pages; aggregate-only; ODPC-registered analytics plan; no PII in analytics store

Physical distribution:
27. FR-M37-013: physical distribution tracking; county coordinator records dispatch with quantity and date; system tracks coverage vs county population; coverage metric per county daily
28. FR-M37-024: county distribution dashboard: Kenya map fill by coverage percentage; drill-through to dispatch records; print pack download button; target >= 90% of county population by T-30

Accessibility and observability:
29. FR-M37-019: WCAG 2.2 AA compliance on all voter-facing content pages (M13 integration and public portal) and CMS authoring UI for officer use
30. FR-M37-030: OpenTelemetry traces exported to central collector; Grafana dashboard eel-iems-37; SLO alert: M13 feed availability >= 99.9%; alert ruleset iems-37-alerts

## Acceptance criteria

1. AC-M37-01: >= 95% fact-check reviews completed within 5 business days; review completion timestamp recorded; Training Director report confirms
2. AC-M37-02: >= 80% of core catalogue available in all 11 languages; language-coverage report exported; the Commission Director sign-off
3. AC-M37-03: M13 content feed availability >= 99.9%; CDN uptime monitor; SLO alert at 99.5% threshold
4. AC-M37-04: physical distribution coverage >= 90% of target counties by T-30; distribution dashboard confirms per-county status
5. AC-M37-05: zero unreviewed content published; automated SpiceDB gate check on every publish action; penetration test confirms reviewer cannot bypass
6. AC-M37-06: WCAG 2.2 AA pass on all voter-facing content pages and CMS authoring UI; independent accessibility audit at OAT gate
7. AC-M37-07: public content feed p99 <= 1 s under 1M requests/day on election day; CDN hit ratio >= 95%; k6 load test confirms
8. AC-M37-08: retract-and-republish workflow completes and M13 receives push notification within 10 minutes of Director issuing retract; integration test with mock M13
9. AC-M37-09: CDN cache purge triggered within 5 minutes of content expiry event; no stale expired content served after purge; chaos test confirms
10. AC-M37-10: infected asset upload rejected by virus scanner before reaching MinIO; zero infected files in object store; security test with EICAR test file
11. AC-M37-11: approved content retained >= 7 years post-election; draft and rejected versions retained >= 2 years; retention schedule audited at OAT gate
12. AC-M37-12: fact-check partner cannot access content outside assigned queue and cannot publish directly; SpiceDB policy enforced; ZAP penetration test confirms no bypass

## Applicable edge cases

data:
- EC-DAT-01: duplicate distribution event submission with same idempotency key; cached response returned; no duplicate push to M13 or WhatsApp store
- EC-DAT-02: same distribution request with different body and same idempotency key; 422 idempotency-key-reuse returned; no action taken; logged
- EC-DAT-09: Somali or Turkana text with extended Unicode characters in article body; ICU NFC normalization applied; no truncation or layout overflow in M13 feed
- EC-DAT-13: published content item soft-deleted via retract action; tombstone honoured; no longer served on M13 feed; version history preserved for audit; purge does not remove version records
- EC-DAT-14: uploaded video asset would produce outbox event > 1 MB; blob stored in MinIO; event carries hash reference only; no oversized outbox event emitted

security:
- EC-SEC-03: HSM unavailable during Director approval signing; approval queued; eel-outbox-rs retains; ingest continues; alert raised; no content published without valid Director signature
- EC-SEC-05: SQL injection or XSS attempt in article body or metadata fields; parameterised queries only; Semgrep CI; ZAP scan; fuzz corpus applied at CI
- EC-SEC-07: external reviewer uses crafted path to access content outside review queue; SpiceDB denies; access attempt logged to audit trail
- EC-SEC-09: rate-limit evasion on public content feed endpoint; keyed by IP/64 subnet; 1000/min enforced; excess requests shed with 429
- EC-SEC-12: CDN cache poisoning attempt via manipulated cache key; response key includes content hash and signature version; client verifies before render
- EC-SEC-16: structured logging strips fact-check reviewer IDs and officer names from unscoped traces; Tier A/B fields absent from telemetry; log-scanner fixture in CI

identity:
- EC-IDN-02: reviewer JWT expires during multi-annotation session; in-flight annotation save completes; next request returns 401 with silent refresh; no annotation lost
- EC-IDN-10: SpiceDB unavailable; all CMS write operations fail closed with 503; CDN-cached public feed continues; alert raised to SRE

scale:
- EC-SCL-01: 1M content requests per day from M13 on election day; CDN edge absorbs; origin p99 <= 1 s; k6 load test with >= 95% CDN hit ratio
- EC-SCL-04: Kafka consumer lag > 60 s on eel.training.content.v1 topic; alert raised; backpressure applied; eel-outbox-rs retains events; no event loss

connectivity:
- EC-CON-01: CDN origin unreachable; M13 falls back to cached content with staleness banner visible to voter; stale but approved content served until origin recovers; no expired content surfaced
- EC-CON-05: flapping connection to WhatsApp Business API; eel-outbox-rs retries with backoff+jitter; no duplicate push; eventual delivery confirmed by WhatsApp receipt

operations:
- EC-OPS-03: WhatsApp Business API approval delayed beyond T-26; SMS broadcast via M13 carries full voter education load; WhatsApp marked supplementary in distribution plan
- EC-OPS-08: MinIO bucket quota reached during video upload; 507 returned to uploader; alert raised to SRE; no partial file stored; quota headroom monitored daily
human:
- EC-HUM-06: WCAG 2.2 AA on M13 voter content feed and CMS authoring UI; screen reader + axe-core + manual test on 3 languages; compliant at OAT gate
- EC-HUM-07: KSL video captions required on all video content items published; caption asset presence checked before item transitions to APPROVED status
- EC-HUM-08: Somali and Turkana long strings in article titles and summaries; no truncation; ICU plurals; font fallback tested at 150 percent pseudo-length
- EC-HUM-09: colour-blind users on M13 voter content feed; all content-category status indicators use symbol + text + colour; not colour alone

legal:
- EC-LEG-01: candidate conduct content published after candidate withdrawal from race; content auto-tagged with expiry on withdrawal event; M13 stops serving within 5 minutes
- EC-LEG-02: court-ordered election postponement; all pre-election content items with expiry on original election date auto-extended by signed calendar amendment; M13 notified

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

- WhatsApp Business API approval: 6-8 week lead time; the Commission must initiate application at T-120; fallback plan (SMS + M13) must be documented before go-live.
- CDN provider: Cloudflare is ADR-forbidden; use Akamai-class or Kenyan CDN (Kenya-peered nodes); confirm provider selection before architecture review.
- Community validators for minority languages: the Commission must contract per-language validators; contract terms and schedules required before translation pipeline activates.
- Fact-check partner rotation list: the Commission must identify and onboard external CSO fact-check partners before CMS goes live; partner accounts are external and scoped read+annotate only.
