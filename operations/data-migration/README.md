# operations/data-migration

One-way migration of voter registration, biometric identification, and results data from
the legacy KIEMS system into eel services. The process is non-destructive: legacy KIEMS
is kept in read-only mode for the 60-day petition window after cutover.

## Overview

KIEMS (Kenya Integrated Election Management System) is the pre-eel system holding
historical BVR, BVI, and RTS data. This migration ingests that data into eel services.
The process is one-way: data flows from KIEMS to eel, not the reverse.

## Scope

| Data type | Source | Target eel service |
|-----------|--------|--------------------|
| Voter registration records | KIEMS BVR PostgreSQL | `bvr-svc` |
| Biometric voter identification records | KIEMS BVI PostgreSQL | `bvi-svc` |
| Historical results | KIEMS RTS PostgreSQL | `rts-svc` |
| Device registrations | KIEMS asset register | `inventory-svc` via kit attestation profiles |

## Order of operations

1. Freeze KIEMS (switch to read-only mode).
2. Take PostgreSQL dump of KIEMS databases.
3. Run extraction scripts (`extract-legacy-voters.sh`, `extract-legacy-kits.sh`).
4. Validate extracted CSV against KIEMS counts (`validate-migration.py`).
5. Ingest extracted data into eel services.
6. Run post-ingest validation (`validate-migration.py --post`).
7. Reconcile: compare pre and post record counts; hash check.
8. Cut over: update DNS and application configuration to point at eel.
9. Keep KIEMS read-only for petition window (60 days post-election).

See `migration-runbook.md` for the full step-by-step commands.

## Usage

```bash
# Step 1: freeze KIEMS (run on KIEMS infrastructure)
ssh kiems-admin@kiems.internal "pg_ctl stop -m fast -D /var/lib/pgsql/data"
# Then restart in read-only mode per KIEMS ops runbook.

# Step 2: extract voter records
bash operations/data-migration/scripts/extract-legacy-voters.sh \
  --kiems-host kiems-db.internal \
  --output /migration/voters.csv

# Step 3: pre-ingest validation
uv run python operations/data-migration/scripts/validate-migration.py \
  --csv /migration/voters.csv \
  --kiems-count <expected-count>

# Step 4: ingest into bvr-svc
curl -X POST \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: text/csv" \
  --data-binary @/migration/voters.csv \
  https://bvr.eel.internal/api/v1/admin/bulk-import

# Step 5: post-ingest validation
uv run python operations/data-migration/scripts/validate-migration.py \
  --post \
  --kiems-count <expected-count> \
  --eel-host https://bvr.eel.internal
```

## Files

| File | Description |
|------|-------------|
| `scripts/extract-legacy-voters.sh` | Reads KIEMS PG dump; emits CSV for bvr-svc ingest |
| `scripts/extract-legacy-kits.sh` | Extracts device registrations; maps to kit attestation profiles |
| `scripts/validate-migration.py` | Compares pre/post counts, hashes, and referential integrity |
| `migration-runbook.md` | Step-by-step order of operations with exact commands |

## Config and environment

| Variable | Description |
|----------|-------------|
| `KIEMS_HOST` | KIEMS database host |
| `KIEMS_PGPASSWORD` | KIEMS PostgreSQL password; from vault |
| `ADMIN_TOKEN` | eel admin bearer token for bulk-import API |
| `EEL_BVR_URL` | `bvr-svc` base URL |
| `EEL_BVI_URL` | `bvi-svc` base URL |
| `EEL_RTS_URL` | `rts-svc` base URL |

## Dependencies

- PostgreSQL 17 client (`pg_dump`, `psql`) on the migration machine
- `uv` and Python 3.11+ for `validate-migration.py`
- Access to KIEMS database (read-only after freeze)
- Admin API access to `bvr-svc`, `bvi-svc`, `rts-svc`

## Test and verify

```bash
# Dry run extraction (no writes; validate record count match)
bash scripts/extract-legacy-voters.sh --dry-run --kiems-host kiems-db.internal

# Run validator against a sample CSV
uv run python scripts/validate-migration.py \
  --csv scripts/sample-voters.csv \
  --kiems-count 100
# Expected: "Validation passed. 100 records. Hash match: OK."
```

## Related docs

- `operations/README.md`: operations overview
- `docs/runbooks/incident-response-p0.md`: P0 response if migration produces data loss
- `services/bvr-svc/README.md`, `services/bvi-svc/README.md`, `services/rts-svc/README.md`

## Limitations

- Migration is one-way and irreversible after cutover. Take a full KIEMS backup and
  verify counts before proceeding.
- KIEMS must remain read-only for the 60-day petition window; do not shut it down.
- Record count reconciliation is the primary correctness check; referential integrity
  checks are partial (voter-to-biometric linkage only; historical results are not cross-referenced).
- The migration runbook is written for a single migration pass; incremental sync is not
  supported. If KIEMS data changes after extraction, re-run from step 2.
