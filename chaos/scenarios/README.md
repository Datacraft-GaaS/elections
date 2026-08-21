<title>Chaos Scenario Catalogue</title>

# Chaos Scenario Catalogue

34 scenarios covering network, compute, storage, identity, and application faults. Each scenario
maps to one or more SLO thresholds from ADR-0009 and a service degradation mode from ADR-0019.

## How to read this table

- **Tool/CRD kind**: the Chaos Mesh CRD or script used to inject the fault
- **Expected result**: observable system behaviour that constitutes a passing run
- Scenarios marked **COMPOSITE** appear in `mesh-experiments/eel-election-day-composite.yaml` or
  `eel-infra-stress-composite.yaml`

## Scenario catalogue

| # | Scenario name | Target service(s) | Tool / CRD kind | Expected result |
|---|---------------|-------------------|-----------------|-----------------|
| 01 | Network partition between bvr-svc and abis-svc | bvr-svc, abis-svc | NetworkChaos (Partition) | bvr-svc enters degraded mode, queues biometric jobs, recovers within 30 s of partition heal; no data loss |
| 02 | DNS failure on edge nodes | biometric-edge-svc, kit | NetworkChaos (DNSChaos) | Edge kits switch to cached DNS entries; transactions continue offline; sync resumes on DNS restore |
| 03 | Kafka broker loss (one of three brokers) | rts-svc, bvr-svc, all producers | PodChaos (pod-kill on kafka-broker-1) | ISR rebalance completes within 60 s; no committed messages lost; consumer lag stays below 5000 |
| 04 | CPU spike on rts-svc during submission window | rts-svc | StressChaos (cpu 95%) | rts-svc auto-scales; p99 latency stays below 2 s; no submissions dropped; HPA fires within 45 s |
| 05 | Memory pressure on portal-svc at peak viewer load | portal-svc | StressChaos (mem 90%) | Portal serves read-only degraded mode; live tally continues streaming; OOM kill triggers restart; recovery within 20 s |
| 06 | Node failure at primary DR site | all services on primary node | PodChaos (node-failure via _pod-kill-template.sh) | Patroni promotes mirror; Kafka leader election; all services reschedule on mirror nodes within 120 s |
| 07 | Disk saturation on transparency-svc | transparency-svc | IOChaos (disk-fill 95%) | transparency-svc enters read-only mode; log entries queued in outbox; disk alarm fires to Alertmanager; data integrity preserved |
| 08 | Clock skew between kit and backend (HLC stress) | biometric-edge-svc, bvr-svc | TimeChaos (skew +/-10 min) | HLC monotonicity maintained; UUIDv7 ordering preserved; skew alarm fires; no submission rejected |
| 09 | Certificate expiry simulation (15-min expiry) | all mTLS peers | HTTPChaos + eel-cert-expiry-15min.yaml | SPIRE rotates SVIDs; zero connection drops during rotation; Envoy hot-reloads cert within 5 s |
| 10 | spireAgent crash | biometric-edge-svc, all SPIRE workloads | PodChaos (pod-kill spire-agent) | Workloads continue using cached SVIDs until reissue; spire-agent restarts and re-attests within 30 s |
| 11 | Cilium network policy drop (misconfigured policy) | boundary-svc, rts-svc | NetworkChaos (network-loss via cilium policy) | boundary-svc blocks unauthorized traffic; legitimate traffic unaffected; Hubble logs policy drops; alert fires |
| 12 | Envoy gateway overload (connection saturation) | envoy-gateway | StressChaos (cpu 85%) + HTTPChaos (high rate) | Rate limiting kicks in; 429 responses returned; upstream services unaffected; gateway recovers on load drop |
| 13 | Ingress rate-limit storm (DDoS simulation) | portal-svc ingress | HTTPChaos (abort 80% requests) | WAF and rate limiter absorb traffic; portal returns 429; tally API unaffected; no crash |
| 14 | Database connection pool exhaustion | rts-svc, bvr-svc | StressChaos + eel-pg-primary-kill-patroni-failover.yaml | pgBouncer queues connections; services wait with backpressure; no connection errors escape to clients |
| 15 | Redis/Valkey eviction surge | portal-svc, rts-svc | eel-valkey-flush.yaml (FLUSHALL simulation) | Cache miss storm handled via DB fallback; latency spike stays below SLO; no data loss |
| 16 | MirrorMaker2 replication lag (primary to mirror) | Kafka MirrorMaker2 | PodChaos (pod-kill mirror-maker) | Mirror site detects lag; alert fires at > 30 s lag; failover reads from primary until MirrorMaker2 restarts |
| 17 | HSM PKCS#11 timeout | transparency-svc (hsm-signer) | eel-hsm-unavailable.yaml | Signing requests queue; timeout after 10 s; signed entries complete once HSM returns; no unsigned entries committed |
| 18 | S3/MinIO latency injection (500 ms added) | all MinIO clients | eel-minio-site-loss.yaml + NetworkChaos (delay 500 ms) | Uploads succeed with retries; presigned URL TTL accounting adjusts; SLO for upload p99 holds |
| 19 | ABIS biometric pipeline stall | abis-svc, queue-svc | eel-pod-kill-abis-svc.yaml + eel-sequencer-stall.yaml | queue-svc buffers jobs; abis-svc restarts and processes backlog; no duplicate deduplication keys |
| 20 | RTS submission burst (80 k events/sec) | rts-svc | StressChaos + load generator (loadtest/profiles/preprod-5x.yaml) | rts-svc sustains 80 k events/s; p99 < 500 ms; Kafka consumer lag recovers within 60 s of burst end |
| 21 | Kit network reconnect loop | biometric-edge-svc, kit | NetworkChaos (intermittent partition 5 s on/off) | Kit persists submissions locally; deduplication prevents duplicates on reconnect; CRDT merge correct |
| 22 | ArgoCD sync failure (out-of-sync manifests) | ArgoCD | PodChaos (pod-kill argocd-application-controller) | ArgoCD restarts and re-syncs; no service outage; drift detected and remediated automatically |
| 23 | Keycloak auth cascade delay (500 ms added) | Keycloak, all OIDC clients | NetworkChaos (delay 500 ms to keycloak) | OIDC token cache absorbs delay; client retries succeed; portal login p99 stays below 3 s |
| 24 | SpiceDB policy evaluation timeout | ecc-svc, bvr-svc | eel-spicedb-down.yaml | Services fall back to cached authz decisions for read operations; writes queue; no privilege escalation; SpiceDB restarts and catches up |
| 25 | OpenBao seal/unseal cycle | transparency-svc, all secret consumers | eel-svid-revocation.yaml + OpenBao seal via API | Secret leases renewed on unseal; no secret exposure in logs; downtime under 60 s |
| 26 | Kafka consumer group rebalance (high churn) | rts-svc consumers | PodChaos (pod-kill 3 consumer replicas simultaneously) | Partitions reassigned within 30 s; no message re-processing past idempotency fence; lag recovers |
| 27 | PostgreSQL primary kill with Patroni failover | all Postgres-backed services | eel-pg-primary-kill-patroni-failover.yaml | Patroni promotes standby within 20 s; services reconnect via PgBouncer; no data loss past synchronous commit |
| 28 | Ceph OSD failure (two of six OSDs) | MinIO backed by Ceph | IOChaos + node drain on Ceph OSD pods | Ceph rebalances; MinIO read/write continues; no object loss; rebalance completes within 10 min |
| 29 | Velero backup job timeout | Velero, MinIO | NetworkChaos (delay 2 s to MinIO during backup) | Backup job retries; incremental backup completes; backup integrity verified by `velero backup describe` |
| 30 | queue-svc CV model crash (ONNX runtime panic) | queue-svc | eel-pod-kill-queue-svc.yaml | queue-svc restarts; in-flight jobs requeueed; CV model reloads; no job permanently lost |
| 31 | AI platform ONNX inference timeout | ai-platform-svc | StressChaos (cpu 100% on GPU node) | Inference requests time out gracefully; circuit breaker opens; fallback response returned; no crash cascade |
| 32 | Transparency log compaction stall | transparency-svc | eel-sequencer-stall.yaml | Sequencer stall detected within 5 s; alert fires; backlog queued; on resume, all entries committed in order |
| 33 | Kyverno admission webhook overload | Kyverno | StressChaos (cpu 90% on kyverno) | Admission requests timeout after 10 s; fail-open policy activates; alert fires; kyverno recovers |
| 34 | Site partition: primary isolated from mirror (COMPOSITE) | all services | eel-site-partition-primary-mirror.yaml | Mirror site takes over reads; writes blocked until quorum restored; no split-brain; consistency preserved |

## Running a single scenario by number

```bash
# Example: scenario 04 (CPU spike on rts-svc)
kubectl apply -f chaos/mesh-experiments/eel-stress-chaos-cpu-mem.yaml
# Watch SLO dashboard: https://grafana.internal.eel/d/eel-slo
kubectl delete -f chaos/mesh-experiments/eel-stress-chaos-cpu-mem.yaml
```

## Related docs

- `chaos/mesh-experiments/README.md` -- YAML file reference
- `verification/chaos-runner/README.md` -- automated orchestration
- `docs/runbooks/` -- per-service degradation runbooks
