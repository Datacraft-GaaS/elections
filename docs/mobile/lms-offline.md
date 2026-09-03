# LMS Offline (M38) — Completion Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/poll-worker-training-lms.md`. Backend: training-api LMS
routes exist (`/api/v1/training/lms/*`). Est: 2 ED.

## Current state

`services/training-svc/apps/lms-offline/` — 12 lib files, codegen'd models,
connectivity-aware sync. **Bug**: app posts `/v1/assessments`
(`assessment_repository.dart:33`) but the API serves
`/api/v1/training/lms/assessments/{assessment_id}/submissions` — path
mismatch means sync can never succeed against the real backend. No pinning,
no auth, 3 locales only.

## Target features (completion)

| # | Feature | FR |
|---|---|---|
| F1 | Fix submission path + request/response shapes to match training-api contract | correctness |
| F2 | Authenticated session (training staff JWT) | spec |
| F3 | Pinning + problem+json handling | standard |
| F4 | Course/enrolment/certificate surfaces (read) + offline course packs | spec |
| F5 | Locales: en/sw + Kenyan set | parity |
| F6 | Sync: batch submissions, idempotent, conflict policy | spec |

## Files

```
lib/features/assessment/assessment_repository.dart   fix path + fromJson to API schema
lib/features/sync/sync_service.dart                  migrate to kit-sdk sync engine + jitter backoff
lib/src/core/api_client.dart                         EelHttpClient via eel-core
lib/src/core/local_db.dart                           keep sqlite (Isar parity optional) — document choice
l10n/                                                full locale set
test/path_contract_test.dart, sync_retry_test.dart, ...
```

## Technical implementation

- Submission: `POST /api/v1/training/lms/assessments/{assessment_id}/
  submissions` with `{answers, worker_id, idempotency_key, submitted_at}`;
  response → mark synced; 409/422 conflict → surface + keep local copy with
  `conflict` state and supervisor resolution screen.
- Offline course packs: signed bundles (same content machinery as poll-clerk
  reference library); assessment unlock requires pack signature valid.

## Algorithms

- Full-jitter retry; EWMA-free simple counters for sync panel.

## Backend touchpoints

- training-api paths (exist); add Pact contract files for the 3 used paths.

## UX / UI / ergonomics

- Current flow kept; add: course library with download states, assessment
  timer + autosave (every answer change journals), submission receipt with
  certificate link when completion recorded; localized strings; offline
  posture ("your attempt will submit automatically").

## Testing

- Unit: path/shape contract (fixture JSON), retry, conflict states.
- Widget: assessment + sync states; locales.
- Integration: offline assessment → reconnect submit dedup; kill mid-typing.
- Contract: Pact for submissions/completions.

## Definition of done

Master §9; path mismatch fixed with contract test pinning it; README
matches real endpoints.