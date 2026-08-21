# Legacy KIEMS data migration

## Overview

KIEMS (Kenya Integrated Election Management System) is the pre-eel system holding historical BVR, BVI, and RTS data. This migration ingests that data into eel services. The process is one-way and non-destructive: KIEMS is kept in read-only mode for the petition window after cutover.

## Scope

| Data type | Source | Target eel service |
|-----------|--------|-------------------|
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

See `migration-runbook.md` for step-by-step commands.

## Files

| File | Description |
|------|-------------|
| `scripts/extract-legacy-voters.sh` | Reads KIEMS PG dump; emits CSV for bvr-svc ingest |
| `scripts/extract-legacy-kits.sh` | Extracts device registrations; maps to kit attestation profiles |
| `scripts/validate-migration.py` | Compares pre/post counts, hashes, and referential integrity |
| `migration-runbook.md` | Step-by-step order of operations |
