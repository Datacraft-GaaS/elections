# Migration runbook: KIEMS to eel

## Prerequisites

- KIEMS PostgreSQL credentials stored in OpenBao at `kv/eel/migration/kiems/pg`.
- eel `bvr-svc`, `bvi-svc`, and `inventory-svc` ingest endpoints are accessible.
- `python3` and `psql` available on the migration host.
- An incident ticket open as `<TICKET>`.
- Two authorised officers present for cutover sign-off.

## Step 1: freeze KIEMS

```bash
# connect to KIEMS primary and set read-only:
export KIEMS_PG_HOST=$(vault kv get -field=host kv/eel/migration/kiems/pg)
export KIEMS_PG_PASSWORD=$(vault kv get -field=password kv/eel/migration/kiems/pg)
psql -h $KIEMS_PG_HOST -U kiems_admin -d kiems \
  -c "ALTER DATABASE kiems SET default_transaction_read_only = on;"
# verify:
psql -h $KIEMS_PG_HOST -U kiems_admin -d kiems \
  -c "SHOW default_transaction_read_only;"
# must return "on"
```

## Step 2: dump KIEMS databases

```bash
pg_dump -h $KIEMS_PG_HOST -U kiems_admin \
  --format=directory --compress=9 \
  -d kiems_bvr -f /migration/dumps/kiems_bvr
pg_dump -h $KIEMS_PG_HOST -U kiems_admin \
  --format=directory --compress=9 \
  -d kiems_bvi -f /migration/dumps/kiems_bvi
pg_dump -h $KIEMS_PG_HOST -U kiems_admin \
  --format=directory --compress=9 \
  -d kiems_rts -f /migration/dumps/kiems_rts
```

## Step 3: extract and transform

```bash
bash operations/data-migration/scripts/extract-legacy-voters.sh \
  /migration/dumps/kiems_bvr /migration/output/voters.csv

bash operations/data-migration/scripts/extract-legacy-kits.sh \
  /migration/dumps/kiems_bvr /migration/output/kits.csv
```

## Step 4: pre-ingest validation

```bash
python3 operations/data-migration/scripts/validate-migration.py \
  --phase pre \
  --kiems-dump /migration/dumps/kiems_bvr \
  --output-csv /migration/output/voters.csv \
  --ticket <TICKET>
# must exit 0; any non-zero exit indicates drift
```

## Step 5: ingest into eel services

```bash
# ingest voters into bvr-svc:
curl -sf -X POST \
  http://bvr-svc.eel-system.svc.cluster.local:8080/admin/import \
  -H "Content-Type: text/csv" \
  --data-binary @/migration/output/voters.csv

# ingest kit profiles into inventory-svc:
curl -sf -X POST \
  http://inventory-svc.eel-system.svc.cluster.local:8080/admin/import/kits \
  -H "Content-Type: text/csv" \
  --data-binary @/migration/output/kits.csv
```

## Step 6: post-ingest validation

```bash
python3 operations/data-migration/scripts/validate-migration.py \
  --phase post \
  --kiems-dump /migration/dumps/kiems_bvr \
  --eel-endpoint http://bvr-svc.eel-system.svc.cluster.local:8080 \
  --ticket <TICKET>
# must exit 0
```

## Step 7: reconcile

```bash
# generate reconciliation report:
python3 operations/data-migration/scripts/validate-migration.py \
  --phase reconcile \
  --kiems-dump /migration/dumps/kiems_bvr \
  --eel-endpoint http://bvr-svc.eel-system.svc.cluster.local:8080 \
  --report /migration/output/reconciliation-<TICKET>.txt
cat /migration/output/reconciliation-<TICKET>.txt
```

## Step 8: cutover (dual-officer)

```bash
# update eel write-gate to open for production traffic:
kubectl patch configmap eel-write-gate -n eel-system \
  --patch '{"data":{"state":"open","migration_ticket":"<TICKET>"}}'

# emit tlog cutover leaf:
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.kiems-migration.cutover.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Step 9: keep KIEMS read-only for petition window

KIEMS remains in read-only mode for 60 days post-election. Do not decommission or modify KIEMS until the petition window has closed and the election result is final.

```bash
# verify KIEMS is still read-only:
psql -h $KIEMS_PG_HOST -U kiems_admin -d kiems \
  -c "SHOW default_transaction_read_only;"
# must still return "on"
```

## Rollback

If post-ingest validation fails, truncate the eel tables and re-run from step 3. KIEMS remains authoritative until cutover; rollback is simply not proceeding to step 8.
