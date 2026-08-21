# eel-abis-bulk-ops

Batch biometric operations for the eel ABIS with idempotent resumption.

## Operation Types

| Type | Description |
|------|-------------|
| enrol | Batch enrolment of new subjects |
| dedup | Identify and flag duplicate subjects above a score threshold |
| delete | Soft/hard delete honouring data retention policy |
| export | Export subject records to CSV |
| rewrap | Re-encrypt templates under a new Key Encryption Key |
| import | Ingest subjects from a watchlist CSV |
| retire | Soft-retire subjects from the active gallery |

## Resumption

Each operation tracks `checkpoint_offset` + counters in `eel_bulk_operations`.
On resume, the worker skips to `checkpoint_offset` and continues from there.
Idempotency: items already processed at a lower offset are not re-processed.

## Failure Manifests

Failed items are collected as `FailureRecord` structs and serialised to a CSV
manifest uploaded to MinIO. The manifest URI is stored in `manifest_uri`.

## Integration Points

- `eel_bulk_operations` table: operation lifecycle and progress tracking.
- Consumer: operations dashboard, operator CLI.
- Workers: each op_type dispatches to a specialised module.

## SLA Impact

Large operations (N>100k) should be scheduled during off-peak hours.
Pause/resume prevents query saturation on the matcher fleet.
