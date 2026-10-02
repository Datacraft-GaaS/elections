# Post-election review (M41)

Module M41 (`post-election-review`) owned by `governance-svc` (crate `review`). Structures the capture, tracking, and public reporting of lessons-learned findings from all electoral stakeholder categories; routes recommendations to implementing owners; manages parliamentary and ODPC referrals; and runs a long-drip attack register with mandatory 6-month silence alerts.

## Contents

1. What M41 does (vs M40 court interface and M25 grievance channels)
2. Data plane (schema)
3. Finding intake sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M41 does (vs M40 court interface and M25 grievance channels)

- **M40** = formal legal evidence disclosure to courts; operates under judicial timelines.
- **M25** = citizen grievance channels during the election period.
- **M41** = structured post-election lessons-learned: captures findings from Commission staff, observers, parties, civil society, and the public; tracks recommendations through owner assignment to closure; publishes a public tracker; manages a long-drip attack register (§XXI.C.20) that alerts if a known attack pattern goes unreviewed for 6 months.

M41 has no legal force on external parties (unlike M40). Its value is institutional memory and public accountability.

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/review/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **LessonsLearningFinding** | Root entity: `finding_id` (UUID7), `election_cycle`, `category` (`TECHNOLOGY / LOGISTICS / LEGAL / SECURITY / TRAINING / COMMUNICATIONS`), `source_type` (`STAFF / OBSERVER / PARTY / CIVIL_SOCIETY / PUBLIC`), `source_ref`, `severity` (`CRITICAL / MAJOR / MINOR / OBSERVATION`), `description_hash`, `status` (`OPEN / UNDER_REVIEW / RECOMMENDATION_ISSUED / CLOSED / REFERRED`), `created_at`. | B |
| **Recommendation** | Action derived from a finding: `recommendation_id` (UUID7), `finding_id`, `owner_team`, `action_description_hash`, `target_date`, `status` (`PENDING / IN_PROGRESS / COMPLETE / DEFERRED`), `evidence_of_completion_s3_key`, `closed_at`. | B |
| **ParliamentReferral** | Referral to Parliament / ODPC: `referral_id` (UUID7), `finding_id`, `referral_body` (`PARLIAMENT / ODPC / DPP / OTHER`), `referral_letter_s3_key`, `submitted_at`, `acknowledged_at`, `outcome`. | B |
| **LongDripAlert** | Long-drip attack register entry: `alert_id` (UUID7), `finding_id`, `attack_pattern_tag`, `last_reviewed_at`, `next_review_due` (last_reviewed_at + 180 days), `silence_alert_sent` (boolean), `status` (`ACTIVE / RESOLVED`). | B |

## 3. Finding intake sources

| Source | Channel | Volume estimate | Auth |
|---|---|---|---|
| Commission staff | LiveView internal form | ~500 per cycle | JWT + SpiceDB `COMMISSION_OFFICER` |
| Domestic observers | Observer portal | ~2,000 per cycle | OIDC (accredited observer ID) |
| International observers | Email import + officer entry | ~200 per cycle | Officer JWT on behalf of |
| Political parties | Party portal | ~300 per cycle | Party rep OIDC |
| Civil society | Public portal form | ~1,000 per cycle | Anonymous or OIDC |
| Public | Public portal form (EN + SW + 5 regional languages) | ~5,000 per cycle | Anonymous |

All free-text inputs are stored as SHA-256 hashes with the plaintext in S3 (`eel-review-findings/`) under WORM retention. The public tracker shows only category + severity + status, never the raw description.

## 4. Pipeline

```
finding submitted via any intake channel
      -> LessonsLearningFinding created (status = OPEN)
      -> if severity = CRITICAL: immediate Slack alert to review coordinator
      -> review coordinator assigns to `UNDER_REVIEW`
      -> analyst drafts Recommendation(s) with owner_team + target_date
      -> status -> RECOMMENDATION_ISSUED
      -> if finding warrants referral:
            -> ParliamentReferral created; letter generated from template
            -> letter S3 key stored; submitted_at recorded
            -> status -> REFERRED
      -> if finding matches attack pattern in long-drip register:
            -> LongDripAlert created with attack_pattern_tag
            -> long_drip_review_scheduler (cron, packages/eel-scheduler-rs):
                  -> checks LongDripAlert.next_review_due daily
                  -> if now() > next_review_due and silence_alert_sent = false:
                        -> send alert to review coordinator + CISO
                        -> silence_alert_sent = true
                  -> coordinator sets last_reviewed_at = now() to reset clock
      -> Recommendation owner marks complete with evidence
      -> if all Recommendations closed: finding status -> CLOSED
      -> public tracker updated in real time via SSE feed (Astro)
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/governance-review.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/review/findings` | Any (anonymous or OIDC) | Submit finding |
| `GET /v1/review/findings` | Commission officer (JWT) | List findings + filter |
| `PATCH /v1/review/findings/{finding_id}` | Review coordinator | Update status / assign |
| `POST /v1/review/findings/{finding_id}/recommendations` | Analyst | Add Recommendation |
| `PATCH /v1/review/recommendations/{rec_id}` | Owner team | Update status + evidence |
| `POST /v1/review/findings/{finding_id}/referrals` | Commissioner | Create ParliamentReferral |
| `GET /v1/review/long-drip` | Review coordinator + CISO | List LongDripAlerts by status |
| `PATCH /v1/review/long-drip/{alert_id}/review` | Review coordinator | Record review; reset clock |
| `GET /v1/review/public-tracker` | Public (60/hr) | Aggregate counts by category + status; no PII |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Review coordinator console (LiveView) | `services/governance-svc/ex/review_web` | Manage findings lifecycle; view long-drip register |
| Public intake form + tracker (Astro) | `services/governance-svc/astro/governance-public` | Citizens submit findings; public views aggregate tracker |
| Commission admin panel (Django) | `services/governance-svc/django/governance-admin` | Commissioner signs referral letters; exports for Parliament |

Languages: EN + SW on staff surfaces. Public intake form: EN + SW + Kikuyu + Luo + Kamba + Kalenjin + Luyia. WCAG 2.2 AA. Public tracker is language-neutral (counts + status labels only).

## 7. Integrity + anti-tamper

- **Immutable finding descriptions** - plaintext stored in S3 WORM; `description_hash` in the DB row ensures no silent edit. A hash mismatch triggers an audit alert.
- **Long-drip silence enforcement** - the `long_drip_review_scheduler` is idempotent (keyed by `alert_id`); duplicate runs are no-ops. If the scheduler itself is silent for >24 hours, a watchdog alert fires to CISO.
- **Referral letter integrity** - `ParliamentReferral.referral_letter_s3_key` points to an S3 WORM object. The letter is generated once; subsequent re-sends use the same object key.
- **Public tracker rate limiting** - `GET /v1/review/public-tracker` is rate-limited to 60/hr per IP to prevent enumeration scraping.
- **Audit shadow** - all writes to `LessonsLearningFinding`, `Recommendation`, and `ParliamentReferral` are shadowed into `eel_governance_audit_shadow`.

## 8. Kenya statute alignment

- **Elections Act §4(b)** - Commission shall promote public confidence in electoral processes; the public tracker operationalises accountability.
- **the electoral authority's establishing Act §5(2)(d)** - Commission shall undertake continuous research into electoral matters; M41 formalises the research intake and tracking mechanism.
- **Kenya Data Protection Act §26 (minimisation)** - only hashes in the DB; raw descriptions in S3 WORM with access restricted to review coordinators and above.
- **Public Finance Management Act** - `ParliamentReferral` with outcome tracking supports the Commission's annual report obligations to Parliament under §§68-70.

## 9. Honest caveats + external prerequisites

- The long-drip attack register's 180-day review interval is a policy choice, not a statutory requirement. the Commission security policy (referenced in the spec) sets this interval; it can be changed via a config value in `governance-svc` environment without a code change.
- Parliamentary referral is a letter-based process. There is no API integration with Parliament. The `acknowledged_at` and `outcome` fields are manually updated by Commission legal staff.
- Anonymous public submissions cannot be followed up. The public tracker shows aggregate data only; individual submission status is not visible to the submitter unless they used OIDC.
- Finding quality depends on observer debrief participation. the Commission must actively solicit debrief reports from M42/M43 accredited observers post-election; the system can ingest them but cannot compel them.

## 10. Test coverage + operational runbooks

Tests under `services/governance-svc/crates/review/tests/`: integration via `testcontainers`, long-drip silence detection property-based tests via `proptest` (time-mocked), S3 WORM hash verification.

Runbooks under `docs/runbooks/`:

- `governance-svc.md` - deploy, rollback, review module configuration.
- `long-drip-scheduler-silent.md` - response procedure when scheduler watchdog fires.
- `referral-letter-generation.md` - manual letter generation if templating fails.

Related documents: spec at `docs/plans/modules/post-election-review.md`; capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`; M40 at `docs/features/election-court-it-interface.md`.

## Edge cases and known gaps

- Anonymous public submissions that later turn out to be coordinated spam: the review coordinator can bulk-mark submissions as `ABANDONED` with an `SPAM` category tag. The raw submission hashes are retained; the public tracker count is unchanged (submitted != reviewed).
- A `LongDripAlert` for a permanently mitigated attack pattern: the coordinator can set `status = RESOLVED` with a detailed `resolution_note_hash` after uploading the mitigation evidence. The scheduler no longer generates silence alerts for resolved entries.
- Findings that surface classified security vulnerabilities: `severity = CRITICAL` findings with a `category = SECURITY` tag are restricted to `COMMISSION_SECURITY` role in SpiceDB; they do not appear in the public tracker aggregate. The public tracker shows the count as `RESTRICTED` rather than omitting it.
- Parliament response tracking: `ParliamentReferral.outcome` is a free-text hash of the parliamentary response. A future integration with the Kenya National Assembly Hansard API is noted as a gap for auto-ingesting parliamentary responses.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*

**Performance targets**: All read endpoints respond within 200ms at p95 under 100 concurrent users. Write endpoints (including HSM-signing paths) respond within 1,000ms at p95. Bulk operations (roster upload, CSV import) are async with a job status poll endpoint.

**SLA targets**: All API reads respond within 200ms p95. Event processing latency from trigger to state update is under 5 seconds under normal load. HSM-signing operations complete within 3 seconds.
