# SpiceDB schema migration

## Context

SpiceDB supports live schema writes without downtime. There is no migration state machine: a `zed schema write` call atomically replaces the current schema with the new one. Because permission checks evaluate the live schema, a bad write immediately affects authorization decisions in production.

The promotion path is: dev validation, dev golden tests, staging, staging golden tests, production. Schema changes must be backwards-compatible (additive only). Additive means adding new relations or permissions that do not alter existing permission logic. Breaking changes (removing or renaming relations, changing permission expressions) require a dual-write migration period where both old and new relations are present simultaneously.

## Schema compatibility rules

- Adding a new relation or permission: safe. Apply directly at any stage. Existing relationships and checks are unaffected.
- Removing a relation or permission: breaking. Any service still calling `Check` or `LookupResources` against the removed relation will receive errors. Before removing: confirm no active relationships use the relation, no service code references it, and no policies depend on it.
- Renaming: not supported atomically. SpiceDB has no `ALTER RELATION` equivalent. The correct pattern is add the new name, migrate all relationship writes and reads to the new name across services, delete all relationships under the old name, then remove the old relation from the schema.
- Changing permission logic (the expression body of a `permission` definition): test carefully. The change takes effect immediately on write and may grant or deny access to resources that were previously accessible. Run targeted `zed check` assertions before and after in staging to confirm the delta is exactly what was intended.

## Pre-conditions

Before beginning:

- The new `.zed` file is committed to `packages/authz-schema/` in git. No uncommitted local modifications.
- Corresponding test cases for the new relations or permissions are added to `packages/authz-schema/tests/<context>_test.yaml`.
- The validate step passes locally (see step 1).
- An ITSM change ticket is open.

## Procedure

### Step 1: validate schema locally

Run the bundled `zed validate` against the schema and its relationship fixture file:

```bash
./tools/spicedb/zed validate \
  --schema packages/authz-schema/<context>.zed \
  --relationships-file packages/authz-schema/tests/<context>_test.yaml
```

A passing run prints `validation succeeded`. Fix any errors before proceeding.

### Step 2: apply to dev SpiceDB

```bash
export SPICEDB_ENDPOINT="localhost:50051"
export SPICEDB_TOKEN="eel-dev-preshared-key"

./tools/spicedb/zed schema write \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  --schema-file packages/authz-schema/<context>.zed
```

Confirm the write succeeded by reading back:

```bash
./tools/spicedb/zed schema read \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  | diff - packages/authz-schema/<context>.zed
```

An empty diff means the server holds exactly the new schema.

### Step 3: run golden tests against dev

```bash
platform/spicedb/schemas/golden-test.sh
```

The script runs `zed check` assertions from `packages/authz-schema/tests/` against the dev endpoint. All assertions must pass. If any fail, the schema change has a logic error: fix it before staging.

### Step 4: apply to staging and run golden tests

```bash
export SPICEDB_ENDPOINT="spicedb.eel.staging:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/staging/spicedb/token)"

./tools/spicedb/zed schema write \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  --schema-file packages/authz-schema/<context>.zed
```

Run golden tests against staging:

```bash
SPICEDB_ENDPOINT="spicedb.eel.staging:50051" \
SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/staging/spicedb/token)" \
platform/spicedb/schemas/golden-test.sh
```

Also run the `iam-seed` verify pass against staging to confirm the authz layer as a whole is coherent:

```bash
export SPICEDB_ENDPOINT="spicedb.eel.staging:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/staging/spicedb/token)"
./target/release/iam-seed verify --env staging
```

### Step 5: apply to production

Only after staging golden tests pass. Use the production SpiceDB endpoint:

```bash
export SPICEDB_ENDPOINT="spicedb.eel.prod:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/prod/spicedb/token)"

./tools/spicedb/zed schema write \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  --schema-file packages/authz-schema/<context>.zed
```

Verify the production schema matches the committed file exactly:

```bash
./tools/spicedb/zed schema read \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  | diff - packages/authz-schema/<context>.zed
```

An empty diff is the only acceptable outcome. If there is a diff, the write may have been rejected or a concurrent write raced. Investigate before declaring success.

### Step 6: verify via iam-seed

```bash
export SPICEDB_ENDPOINT="spicedb.eel.prod:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/prod/spicedb/token)"

./target/release/iam-seed verify --env prod
```

All checks must pass. If any fail, the schema change has broken a policy assumption that was not caught by the golden tests. See rollback.

## Rollback

SpiceDB does not support schema rollback natively. Rollback is a forward write of the previous schema.

If relationships were not written for the new relations during the migration window, rollback is straightforward:

```bash
export SPICEDB_ENDPOINT="spicedb.eel.prod:50051"
export SPICEDB_TOKEN="$(bao kv get -field=value kv/eel/prod/spicedb/token)"

git show HEAD~1:packages/authz-schema/<context>.zed > /tmp/<context>-prev.zed

./tools/spicedb/zed schema write \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  --schema-file /tmp/<context>-prev.zed
```

If relationships were written for a relation that no longer exists in the rollback target schema, SpiceDB will reject the schema write. Delete the orphaned relationships first using the `DeleteRelationships` API (see breaking changes below), then apply the old schema.

## Breaking changes: relation removal procedure

Removing a relation from the schema requires eliminating all data that references it first. Follow this sequence precisely.

a. Mark the relation deprecated in the `.zed` file with a comment. Commit and apply the schema with the deprecation comment but the relation still present.

b. Identify all services that write or read the deprecated relation:

```bash
grep -r "<relation_name>" services/
```

Review each match. Update service code to stop writing new relationships for the deprecated relation and to stop calling `Check` against it. Deploy updated services.

c. Confirm no new relationships are being written. Check the SpiceDB write audit log or instrument a `LookupResources` call:

```bash
./tools/spicedb/zed relationships read \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  --subject-type "<subject_type>" \
  --relation "<relation_name>" \
  | head -20
```

d. Delete all existing relationships of the deprecated relation type. Use the SpiceDB `DeleteRelationships` gRPC method via the REST gateway or a purpose-built migration script. Example using the gRPC gateway:

```bash
grpcurl -plaintext \
  -H "Authorization: Bearer $SPICEDB_TOKEN" \
  -d '{"relationship_filter":{"resource_type":"<resource_type>","optional_relation":"<relation_name>"}}' \
  "$SPICEDB_ENDPOINT" \
  authzed.api.v1.PermissionsService/DeleteRelationships
```

Confirm zero relationships remain:

```bash
./tools/spicedb/zed relationships read \
  --endpoint "$SPICEDB_ENDPOINT" \
  --token "$SPICEDB_TOKEN" \
  --subject-type "<subject_type>" \
  --relation "<relation_name>" \
  | wc -l
```

Expected output: `0`.

e. Remove the relation definition from the `.zed` file. Commit. Apply to dev, run golden tests, apply to staging, run golden tests, apply to production following the standard procedure above.

## Trust domain references

| Environment | SpiceDB endpoint | Token path in OpenBao |
|---|---|---|
| dev | `localhost:50051` | static `eel-dev-preshared-key` |
| staging | `spicedb.eel.staging:50051` | `kv/eel/staging/spicedb/token` |
| production | `spicedb.eel.prod:50051` | `kv/eel/prod/spicedb/token` |

The trust domain for dev is `eel.local`. For production it is `eel.prod`. Schema files in `packages/authz-schema/` are trust-domain-agnostic: the endpoint and token determine which domain receives the write.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-spicedb-down.yaml`

The SpiceDB down scenario verifies that a schema migration can be safely rolled back when the SpiceDB instance becomes temporarily unavailable during the promotion window.
