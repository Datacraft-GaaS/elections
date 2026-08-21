# platform/valkey/

Valkey (Redis-compatible) cache deployment for the eel IEMS. Used for session tokens, rate-limit counters, short-lived idempotency keys, and BVI/RTS in-memory state during election day.

Valkey is the Linux Foundation fork of Redis 7, chosen for its BSD license and long-term community support (ADR-0019).

## Contents

| Path | Description |
|---|---|
| `local/` | Kustomize overlay for kind (single instance, no persistence) |

Per-service Valkey resources (logical database assignments, password references) live in `services/<svc>/deploy/overlays/`.

## Logical database assignments

| DB index | Used by | Purpose |
|---|---|---|
| 0 | portal-svc | Session tokens |
| 1 | rts-svc | Result submission idempotency keys |
| 2 | bvi-svc | Live biometric session state |
| 3 | kong | Rate-limit counters |
| 4 | boundary-svc | Geo-fence cache |
| 5-15 | Reserved | Future service use |

## Install

```bash
# Valkey via Bitnami chart:
helm upgrade --install valkey oci://registry-1.docker.io/bitnamicharts/valkey \
  --namespace eel-cache --create-namespace \
  --set auth.existingSecret=valkey-secret \
  --set replica.replicaCount=2
```

## Connection pattern

Services connect via the `eel-resilience-*` Valkey client wrapper, which handles:
- Automatic reconnection with exponential backoff
- Circuit breaking on sustained timeouts
- Read-through to replica instances for non-critical reads

## Persistence

Production: AOF (append-only file) with `everysec` fsync. Data survives pod restarts.
Local: no persistence; cache is ephemeral.

## Verify

```bash
kubectl exec -n eel-cache valkey-master-0 -- valkey-cli PING
# Expected: PONG

kubectl exec -n eel-cache valkey-master-0 -- valkey-cli INFO replication
```

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: standalone, no auth, no persistence |

## Operator responsibilities

- Valkey password is rotated via OpenBao dynamic secrets (KV v2 path `kv/eel/valkey/password`).
- Memory limit is set to 4 GB per instance. Monitor `valkey_memory_used_bytes` in Grafana.
- During election day, BVI session state in DB 2 must survive a Valkey pod restart. Verify AOF is enabled before election day.

## Related docs

- `platform/openbao/README.md`: password injection
- `platform/observability/README.md`: Valkey metrics dashboard
- `docs/runbooks/cache-flush.md`: safe cache flush procedure (without affecting election state)
