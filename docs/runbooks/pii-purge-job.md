# PII purge job

## Purpose

Execute a scheduled or mandated deletion of personally identifiable information (PII). PII is classified as Restricted or Sensitive per the data dictionary; deletion must be complete, auditable, and reversible only through the transparency log. This runbook covers the purge job, verification of deletion, and the audit record.

## Prerequisites

- The purge job CLI or service endpoint available.
- The data dictionary at `packages/data-dictionary/` identifies which fields are PII.
- An incident ticket open as `<TICKET>` (purge tickets are pre-approved).
- Data Protection Officer (DPO) approval recorded in the ticket for mandated purges.

## Commands

### Step 1: confirm the purge scope

```bash
# list the PII fields and the retention policy:
grep -rn "retention\|purge\|PII" packages/data-dictionary/ | head -20
# confirm the scope matches the mandate or schedule
```

### Step 2: dry-run the purge

```bash
# run the purge in dry-run mode to count affected records:
eel-purge --scope <SCOPE> --dry-run --ticket <TICKET>
# record the count of records that would be deleted
```

### Step 3: execute the purge

```bash
eel-purge --scope <SCOPE> --ticket <TICKET>
# the job deletes PII fields and writes a purge event to the outbox
```

### Step 4: verify deletion

```bash
# confirm the PII fields are gone:
psql -d eel -c "SELECT count(*) FROM <TABLE> WHERE <PII_FIELD> IS NOT NULL;"
# must return 0 for the purged scope
# confirm the purge event reached the transparency log:
curl -sf http://transparency-svc.eel-system.svc.cluster.local:9001/admin/leaves \
  | grep '<PURGE_EVENT_ID>'
```

### Step 5: verify no PII in logs or backups

```bash
# run the log-scanner to confirm no PII leaked into logs:
tools/completeness/scan --pii
# confirm the purge did not leave PII in the current backup:
# (the next backup after the purge will be clean; note this in the ticket)
```

### Step 6: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.pii-purge.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"scope\":\"<SCOPE>\",\"records_purged\":\"<N>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. The purged PII fields return 0 rows for the scope.
2. The purge event is present in the transparency log.
3. The log-scanner reports no PII in logs.
4. The ticket records the DPO approval and the record count.
5. Transparency log contains the purge-completed leaf.

## Rollback procedure

PII deletion is not reversible by design. The only recovery is from a pre-purge backup, which reintroduces the PII and must itself be purged. If a purge was executed in error, restore the affected table from the pre-purge backup, then re-run the purge with the correct scope. Confirm with the DPO before any restore.

## Escalation contacts

- Data Protection Officer (contact via encrypted comms)
- Database Lead (on-call via PagerDuty rotation)
- Site Reliability Lead (on-call via PagerDuty rotation)

## Estimated duration

30 to 60 minutes depending on scope size.

## Chaos-drill reference

This is a scheduled procedure, not a chaos scenario. It is related to `docs/runbooks/backup-restore-drill.md` (restore path) and the data dictionary at `packages/data-dictionary/`.
