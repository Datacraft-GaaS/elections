# platform/spicedb/

SpiceDB fine-grained authorization configuration for the eel IEMS. Implements relationship-based access control (ReBAC) across all 18 services using Zanzibar-style relationship graphs.

## Contents

| Path | Description |
|---|---|
| `config/` | SpiceDB deployment configuration and zookie propagation policy |
| `schemas/` | Schema apply and golden-test scripts |
| `local/` | Kustomize overlay for kind (in-memory datastore) |

## Schema management

The authoritative schema lives in `packages/authz-schema/*.zed`. The `schemas/` directory contains helper scripts:

```bash
# Apply the combined schema to a running SpiceDB:
bash platform/spicedb/schemas/schema-apply.sh

# Run golden tests (persona-level assertion checks):
bash platform/spicedb/schemas/golden-test.sh
```

Golden tests assert that each persona (voter, RO, observer, party agent, admin) can and cannot perform specific actions. They gate G1 and G2.

## Zookie propagation

SpiceDB uses zookies (consistency tokens) to prevent stale reads after relationship writes. The propagation policy (in `config/`) sets:
- Minimum zookie revision: `fully_consistent` for election-critical paths
- `at_least_as_fresh` for read-heavy paths (voter portal, observer views)

## Apply schema

```bash
# Via tools/spicedb wrapper:
./tools/spicedb/zed schema write --schema packages/authz-schema/combined.zed \
  --endpoint localhost:50051 --token "$SPICEDB_PSK"

# Via iam-seed:
iam-seed spicedb
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: memory datastore, no persistence, fast startup |

## Operator responsibilities

- Schema changes are backward-compatible only (additive). Removing a relation requires a migration ADR.
- Each `*.zed` file has a corresponding golden test in `packages/authz-schema/<ctx>_test.yaml`. Tests must pass before schema apply.
- The SpiceDB datastore uses PostgreSQL (`eel-db` namespace). Backups are via pgBackRest.

## Related docs

- `tools/spicedb/README.md`: host-side `zed` CLI wrapper
- `tools/iam-seed/README.md`: idempotent schema seeding
- `packages/authz-schema/README.md`: schema organization by context
- `platform/opa-bundles/README.md`: OPA for decisions not expressible in SpiceDB

---

## Technical Explanation (Extended) — spicedb

**Role**: spicedb is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide spicedb-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: spicedb must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from spicedb components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
