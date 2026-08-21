# eel-governance-reporting

M58 Parliamentary and judiciary reporting: statutory draft versioning, testimony transcripts, sign-off workflows.

## What it does

Manages election post-mortem reporting to Parliament (quarterly post-election statutory reports) and the Judiciary (evidence summaries for election petitions). Tracks draft versions, marshals testimony from county election observers and international observers, manages sign-off workflows (county_returning_officer -> national_director -> minister approval for parliament reports), and publishes final reports via transparency-svc.

Fits in governance-svc as the reporting workflow engine. Runs on port 8094; consumes field-mobile.attendance.checked-in and intl-observer.briefing.completed events to build testimony aggregates.

## Public API

`eel_governance_reporting::report::StatutoryReport` aggregate: `fn add_testimony(&mut self, testimony: Testimony) -> Result<()>`, `fn draft_version(&mut self, content: RichText) -> Result<Version>`, `fn request_signoff(&mut self, reviewer_id: str, role: ReviewerRole) -> Result<()>`, `fn publish(&mut self) -> Result<()>`.

Endpoints:
- `GET /v1/governance/parliament-reporting/{id}/draft` - fetch latest draft
- `POST /v1/governance/parliament-reporting/{id}/draft` - create version
- `GET /v1/governance/parliament-reporting/{id}/draft/{version}` - fetch specific version
- `POST /v1/governance/parliament-reporting/{id}/request-signoff` - trigger workflow
- `GET /v1/governance/parliament-reporting/{id}/published` - fetch final (immutable)

Drafts support collaborative editing via operational transformation (OT) library; all versions stored in audit trail.

## Feature flags

`#[cfg(feature = "ot-sync")]` for real-time collaborative draft editing via WebSocket; tests use in-memory version storage.

## Test commands

```bash
cargo test -p eel-governance-reporting --all-features
cargo test -p eel-governance-reporting --lib version_merge_conflict
cargo test -p eel-governance-reporting --test integration_reporting -- --include-ignored
```

Property-based tests verify OT merge semantics; test sign-off workflow state machine.

## Bench commands

```bash
cargo bench -p eel-governance-reporting merge_100_concurrent_edits
```

Merging 100 concurrent draft edits should complete in < 50ms.

## Dependencies of note

- `uuid6`: report and testimony IDs
- `automerge`: OT library for concurrent draft editing
- `chrono`: version timestamps
- `serde_json`: testimony and draft content parsing

## Related docs

- `docs/api-reference.md#reporting` (sign-off workflow diagram)
- `docs/architecture.md#ot-merge` (OT algorithm explanation)
- `migration_008_reporting.sql` (schema: reports, testimonies, draft_versions, signoff_requests)
