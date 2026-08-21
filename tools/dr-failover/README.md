# eel-dr-failover

Automated DR failover assist for the eel IEMS. Measures current RTO/RPO against targets, plans a failover sequence, and executes it step by step with confirmation gates.

Wave context: WP-5.9 (chaos and DR). Used during scheduled DR drills and unplanned outages.

## Install

```bash
cargo build --release -p eel-dr-failover
# Binary at target/release/eel-dr-failover
```

## Subcommands

| Subcommand | Description |
|---|---|
| `measure` | Query Prometheus and Loki to compute current RTO and RPO |
| `plan` | Emit a sequenced failover plan without executing it |
| `execute` | Execute the plan with interactive confirmation at each step |
| `status` | Show the current DR state: primary up, mirror lag, last backup |
| `drill` | Run a full DR drill in simulation mode (no real traffic affected) |

## Usage

```bash
# Check current RTO/RPO posture:
eel-dr-failover measure

# Generate a failover plan to the mirror site:
eel-dr-failover plan --from primary --to mirror

# Execute the plan (prompts before each destructive step):
eel-dr-failover execute --from primary --to mirror

# Non-interactive (CI drill, simulation only):
eel-dr-failover drill --from primary --to mirror --simulate

# Show replication lag and backup age:
eel-dr-failover status
```

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `EEL_PROMETHEUS_URL` | `http://localhost:9090` | Prometheus endpoint |
| `EEL_PRIMARY_KUBECONFIG` | `~/.kube/config` | kubeconfig for the primary cluster |
| `EEL_MIRROR_KUBECONFIG` | (required for failover) | kubeconfig for the mirror cluster |
| `EEL_DR_CONFIRM` | `interactive` | Set to `auto` to skip confirmation prompts (drills only) |
| `EEL_RTO_TARGET_MINUTES` | `30` | RTO target in minutes for pass/fail reporting |
| `EEL_RPO_TARGET_MINUTES` | `15` | RPO target in minutes |

## Exit codes

- `0`: success or drill passed
- `1`: RTO/RPO target missed
- `2`: failover execution error (step failed)
- `3`: missing kubeconfig or Prometheus unreachable

## Failover sequence (abbreviated)

1. Verify mirror replication lag is within RPO.
2. Quiesce writes on primary (Envoy Gateway route weight to 0).
3. Promote mirror PostgreSQL to primary.
4. Update Kafka MirrorMaker2 direction.
5. Point DNS to mirror (Cilium BGP or external DNS record).
6. Re-enable writes.
7. Validate services on mirror.
8. Record failover event to transparency log.

Full sequence is in `docs/runbooks/dr-failover.md`.

## Related docs

- `docs/runbooks/dr-failover.md`: complete runbook with RACI
- `platform/mirrormaker2/README.md`: Kafka replication config
- `platform/pgbackrest/README.md`: PostgreSQL backup and PITR
- `platform/velero/README.md`: cluster-state backup and restore
