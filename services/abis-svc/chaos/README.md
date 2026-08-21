# abis-svc Chaos Scenarios

Chaos Mesh scenarios scoped to `abis-svc`. Apply with:

```
kubectl apply -f services/abis-svc/chaos/<scenario>.yaml
```

Do not apply to `chaos/scenarios/` (top-level project chaos directory).

## Scenarios

### eel-abis-vendor-latency.yaml
Target: `abis-vendor` Python service (`:8090`).
Injects 500ms (+/-50ms jitter) network latency on all traffic from `abis-api` to `abis-vendor` for 10 minutes.
Expected behaviour: circuit-breaker trips after 5 consecutive slow calls; `abis-api` returns HTTP 503 with `{"status":"degraded","reason":"vendor-timeout"}`. No data is lost; requests queue in Kafka outbox.
Alert: `AbisVendorRouterAllUnhealthy` fires within 90s if no vendor recovers.

### eel-abis-vendor-failure.yaml
Target: `abis-vendor` pod.
Kills one `abis-vendor` pod abruptly. Validates that the vendor-router in `abis-matcher` promotes a secondary endpoint or parks requests in the vendor-retry queue until the pod recovers.
Alert: `AbisVendorRouterAllUnhealthy`.

### eel-abis-pg-primary-loss.yaml
Target: `postgres` primary pod.
Partitions network from Postgres primary to `abis-api` pods for 8 minutes. Validates Patroni failover, PgBouncer re-routing, and SKIP LOCKED semantics on outbox resumption. RPO <= 5 min.
Alert: (Postgres-layer alert in platform alerts).

### eel-abis-shard-loss.yaml
Target: `abis-matcher` tier-2 shard replica.
Kills 1 of 3 shard replicas. Validates that queries route to the remaining 2, HPA scales out, and no match request errors.
Alert: (queue_depth HPA trigger).

### eel-abis-adjudicator-queue-flood.yaml
Target: `abis-api` adjudication endpoint.
Injects 100k case-create requests in 60s via sidecar. Validates priority queue serving `ElectionDayHold` first and SLA timer firing.
Alert: `AbisEnrolmentBacklog`, `AbisAdjudicationSlaBreach`.

### eel-abis-attest-tdx-nonce-replay.yaml
Target: `abis-api` match endpoints via HTTP chaos.
Replays a stale attestation nonce (>60s old) on all incoming match requests. Validates rejection with HTTP 401 and security event emission.
Alert: `AbisAttestationFailureSpike`.

### eel-abis-hnsw-memory-pressure.yaml
Target: `abis-matcher` pods.
Applies 8 GiB memory stress across 4 workers for 10 minutes. Validates HPA scale-out and recall@10 >= 0.95 maintained.
Alert: (HPA-driven; Grafana HNSW panel shows index size and scale).
