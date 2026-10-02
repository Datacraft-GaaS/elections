# Voter education CMS (M37)

Module M37 (`voter-education-cms`) owned by `training-svc`. Manages the authoritative catalogue of voter education content across 11 Kenyan languages, enforces fact-checking review before publish, tracks content expiry, and feeds the M13 multi-channel distribution pipeline (SMS, IVR, Astro web, kiosk PDF).

## Contents

1. What M37 does (vs M38 LMS and M13 distribution)
2. Data plane (schema)
3. Content lifecycle
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M37 does (vs M38 LMS and M13 distribution)

- **M13** = multi-channel distribution engine (SMS bulk, IVR, web, kiosk); it consumes published content but does not author it.
- **M37** = content authoring, versioning, fact-check review, expiry scheduling, and language catalogue for voter education materials. Authors draft here; M13 pulls approved items.
- **M38** = poll-worker training LMS; uses a separate curriculum from M37 but the same `ContentItem` + `ContentVersion` aggregates for course materials.

M37 is the single source of truth for what citizens should be told and in which language. It does not send anything itself; it publishes `eel.training.content_published.v1` events that M13 subscribes to.

## 2. Data plane

Four aggregates (per `services/training-svc/crates/education-cms/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ContentItem** | Root entity: `item_id` (UUID7), `slug`, `category` (`VOTING_PROCESS / REGISTRATION / RIGHTS / RESULTS / SAFETY`), `languages_available[]`, `status` (`DRAFT / FACT_CHECK / APPROVED / PUBLISHED / EXPIRED`), `expires_at`, `canonical_language` (`sw`). | B |
| **ContentVersion** | Immutable snapshot of one language variant: `version_id` (UUID7), `item_id`, `language_code` (BCP-47), `body_sha256`, `body_s3_key`, `rendered_formats[]` (`SMS_160 / IVR_AUDIO / WEB_HTML / KIOSK_PDF`), `created_by`, `created_at`. | B |
| **FactCheckReview** | Fact-check assignment: `review_id` (UUID7), `version_id`, `reviewer_id`, `assigned_at`, `verdict` (`PASS / FAIL / CONDITIONAL`), `conditions_text_hash`, `closed_at`. | B |
| **DistributionRelease** | Snapshot of items released to M13 per distribution cycle: `release_id` (UUID7), `cycle_tag`, `item_ids[]`, `released_at`, `m13_ack_id`. | B |

Language codes supported: `sw` (canonical), `en`, `ki` (Kikuyu), `luo`, `kam` (Kamba), `kal` (Kalenjin cluster), `meru`, `luy` (Luyia cluster), `gir` (Giriama), `som` (Somali), `mas` (Maasai). ISO 639-3 codes where available, BCP-47 private-use tags where not.

## 3. Content lifecycle

| Stage | Actor | Gate |
|---|---|---|
| DRAFT | Content author (Commission Education Dept) | None; autosave on every edit |
| FACT_CHECK | Education dept head | Assign `FactCheckReview`; at least one reviewer required |
| APPROVED | Fact-checker | Verdict = PASS or CONDITIONAL (with conditions resolved) |
| PUBLISHED | System (on approval) | Emit `eel.training.content_published.v1`; M13 picks up |
| EXPIRED | System (on `expires_at` crossing) | Emit `eel.training.content_expired.v1`; M13 removes from active pool |

Content expiry is evaluated by the `content_expiry_sweep` cron job (idempotent, `packages/eel-scheduler-rs`). An item approaching expiry within 14 days triggers a renewal reminder to the content author via email.

## 4. Pipeline

```
author drafts ContentItem + ContentVersion (per language)
      -> upload body to S3 (eel-education-content/)
      -> body_sha256 stored; S3 object-lock for 7 years (Elections Act archive)
      -> dept head assigns FactCheckReview
      -> fact-checker reviews each language variant
            -> PASS: all variants approved
            -> FAIL: item returns to DRAFT; author notified
            -> CONDITIONAL: author resolves conditions; re-review
      -> on APPROVED:
            -> render ContentVersion to SMS_160 / IVR_AUDIO / WEB_HTML / KIOSK_PDF
               via rendering workers (packages/eel-render-rs)
            -> status -> PUBLISHED
            -> emit eel.training.content_published.v1
            -> M13 distribution pipeline ingests and schedules
      -> on expires_at crossing:
            -> content_expiry_sweep -> status -> EXPIRED
            -> emit eel.training.content_expired.v1
            -> M13 removes item from active pool
      -> quarterly DistributionRelease snapshot -> m13_ack_id stored
```

## 5. API surface

Contract: `packages/openapi-contracts/training/education-cms.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/education/items` | Auth user | List content items, filter by status / language / category |
| `POST /v1/education/items` | Content author | Create ContentItem |
| `GET /v1/education/items/{item_id}` | Auth user | Retrieve item + versions |
| `POST /v1/education/items/{item_id}/versions` | Content author | Add ContentVersion for a language |
| `POST /v1/education/items/{item_id}/fact-check` | Dept head | Assign FactCheckReview |
| `PATCH /v1/education/items/{item_id}/fact-check/{review_id}` | Fact-checker | Submit verdict |
| `POST /v1/education/items/{item_id}/publish` | System / dept head | Trigger publish (APPROVED guard) |
| `GET /v1/education/items/{item_id}/versions/{version_id}/render` | Internal (M13) | Pull rendered format payload |
| `GET /v1/education/releases` | M13 system (SVID) | List DistributionReleases for cycle |
| `GET /v1/education/items/public` | Public (30/hr) | Browse published items (no body; metadata only) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| CMS authoring console (LiveView) | `services/training-svc/ex/cms_web` | Authors draft and manage ContentItems; dept head assigns reviews |
| Fact-check review panel (LiveView) | `services/training-svc/ex/cms_web` | Fact-checkers see queue, review evidence, record verdict |
| Content calendar (Django) | `services/training-svc/django/education_admin` | Ops staff manage expiry schedules, distribution cycles |
| Public education portal (Astro) | `services/training-svc/astro/voter-education` | Citizens browse published items in their language |

Languages: EN + SW on staff surfaces. Public portal: all 11 catalogue languages. WCAG 2.2 AA. Audio versions served via IVR_AUDIO pre-renders.

## 7. Integrity + anti-tamper

- **Immutable versions** - `ContentVersion` rows are insert-only; no UPDATE on body or SHA-256. A new edit creates a new version row.
- **S3 object-lock** - body files stored with WORM retention (Elections Act 7-year archive rule). Deletion requires dual-admin approval logged in `eel_bvr_audit_shadow`.
- **Fact-check chain** - no item can transition to PUBLISHED without a `FactCheckReview.verdict = PASS`. The gate is enforced server-side; the UI cannot bypass it.
- **SHA-256 body fingerprint** - every `ContentVersion` stores `body_sha256`; the render worker verifies the hash before rendering. A mismatch aborts and alerts ops.
- **Event chain** - `eel.training.content_published.v1` carries `item_id + version_ids[] + body_sha256[]`; M13 verifies hashes before distribution.
- **Expiry precision** - `expires_at` is stored in UTC; the sweep job uses a server-side clock (PostgreSQL `now()`) to avoid client-clock skew.

## 8. Kenya statute alignment

- **Elections Act §44** - Commission has a statutory duty to conduct voter education; M37 operationalises content governance for that mandate.
- **Elections (Technology) Regulations 2017** - digital content must be versioned and auditable; M37 satisfies via immutable `ContentVersion` + SHA-256 chain.
- **Kenya Data Protection Act §26 (minimisation)** - no voter PII in CMS records; language preference is derived from ward geography, not stored per-citizen.
- **Official Languages Act 2023** - Kiswahili is canonical; all items require a `sw` variant before any other language version is approved.
- **Broadcasting Act §34** - fact-check review process aligns with the statutory accuracy requirement for public information campaigns.

## 9. Honest caveats + external prerequisites

- Audio rendering (IVR_AUDIO format) requires a TTS engine. The adapter in `integrations/tts-engine/` targets a self-hosted Kokoro TTS (Apache-2.0) instance on the ml/vault server. If TTS is unavailable, audio format is skipped and M13 falls back to SMS-only for IVR channels.
- 11-language fact-checking requires domain experts in each language. the Commission must contract reviewers for the five minority-language variants (Kalenjin, Meru, Luyia, Giriama, Somali, Maasai). The system supports them; the human pool is an external dependency.
- Kiosk PDF rendering assumes A5 print stock and requires the kiosk PDF template (under `services/training-svc/templates/kiosk/`) to be localised per language by a graphic designer before launch.
- M13 integration uses `eel.training.content_published.v1` via Kafka. If M13 is unavailable, published content sits in the outbox until M13 reconnects; no fallback SMS blast is triggered automatically.

## 10. Test coverage + operational runbooks

Tests under `services/training-svc/crates/education-cms/tests/`: property-based via `proptest` on lifecycle state machine, integration via `testcontainers` for PG + S3 + Kafka, rendering smoke tests per format type.

Runbooks under `docs/runbooks/`:

- `training-svc.md` - deploy, rollback, TTS adapter failure mode.
- `content-expiry-sweep.md` - manual trigger + idempotency verification.
- `s3-object-lock-admin.md` - dual-admin deletion protocol for WORM-locked content.

Related documents: spec at `docs/plans/modules/voter-education-cms.md` (if present); capability catalogue at `services/training-svc/docs/CAPABILITIES.md`; M13 distribution spec at `docs/plans/modules/`.

## Edge cases and known gaps

- A `ContentItem` whose canonical `sw` variant fails fact-check while minority-language variants have already passed: the item stays in `FACT_CHECK` state and all passing variants are held. Passing variants cannot be partially released; the entire item must pass before any variant is published.
- A language variant whose TTS render fails: the `IVR_AUDIO` format is omitted from `rendered_formats[]` and M13 distributes the available formats only. An ops alert is raised for the failed language-format combination.
- Content imported from a previous election cycle: a `ContentVersion` from cycle N can be copied (new `version_id`, new `item_id`, same body S3 key) and re-submitted for fact-check in cycle N+1. The S3 object-lock countdown is reset from the new `created_at`.
- ODPC data-map note: `ContentItem` and `ContentVersion` rows contain no personal data; they are excluded from the ODPC Article 31 data-map. The `FactCheckReview.reviewer_id` is a Commission officer UUID linked to the HR system; it is in-scope for the staff data-map.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*
