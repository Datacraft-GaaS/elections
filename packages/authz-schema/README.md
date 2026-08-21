# authz-schema

Canonical SpiceDB `.zed` schema files per bounded context, with golden tests per persona. The single source of truth for all authorization relationships in the platform.

## Purpose in context

Every `SpiceDbClient.check()` call across all three language tiers validates a relationship defined in these schemas. Changes to authorization rules are made here first, reviewed, golden-tested, and then deployed to SpiceDB via the migration tooling. This prevents authorization drift between what the code assumes and what SpiceDB enforces.

## Layout

```
authz-schema/
  abis.zed
  ai_platform.zed
  boundary.zed
  bvi.zed
  bvr.zed
  crms.zed
  custody.zed
  cvr.zed
  ecc.zed
  field_mobile.zed
  gazette.zed
  governance.zed
  grievance.zed
  iab.zed
  inspection.zed
  inventory.zed
  party.zed
  portal.zed
  queue.zed
  rts.zed
  support_admin.zed
  training.zed
  transparency.zed
  vprs.zed
  tests/
    <context>/
      <persona>.golden.yaml   # expected decisions per persona
```

## Schema conventions

- One `.zed` file per bounded context; contexts match `packages/openapi-contracts/<ctx>/`
- Every resource type includes a `view` relation for SpiceDB lookup queries
- Permissions follow `<action>_<resource>` naming: `submit_form34a`, `read_voter_record`
- Caveated permissions (time-fenced operations) use SpiceDB caveats

## Deploying schema changes

```bash
# Validate all schemas against SpiceDB:
just validate-authz

# Run golden tests against the local SpiceDB instance:
just test-authz

# Deploy to the cluster (requires SpiceDB migration token):
just deploy-authz --env staging
just deploy-authz --env production
```

## Adding a new bounded context

1. Create `authz-schema/<ctx>.zed` following the existing patterns
2. Add golden test files under `tests/<ctx>/`
3. Run `just validate-authz && just test-authz`
4. Wire the context name into `packages/eel-auth-rs/src/spicedb.rs` check calls
5. Add the context to the Spectral ruleset in `packages/openapi-contracts/.spectral.yaml`

## Verify

```bash
# Check all schemas parse:
zed schema validate authz-schema/*.zed

# Run all golden tests:
just test-authz
```

## Related docs

- `docs/adr/0004-iam.md` - authorization model and SpiceDB role design
- `packages/eel-auth-rs/README.md` - SpiceDB client used by Rust services
- `packages/eel-auth-py/README.md` - SpiceDB client used by Python services
- `packages/eel-auth-ex/README.md` - SpiceDB client used by Elixir services

## Limitations

SpiceDB schema changes are applied via a migration that atomically swaps the schema. There is no incremental migration; a bad schema deployment requires a rollback to the previous schema version. Always run `just test-authz` in CI before merging schema changes.
