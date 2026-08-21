<title>Chaos Mesh Experiment YAML Library</title>

# Chaos Mesh Experiment YAML Library

Reusable Chaos Mesh CRD manifests for the elections platform. All experiments target the
`chaos-testing` namespace and use label selectors to scope blast radius.

## Conventions

- All files are named `eel-<fault-type>-<target>.yaml`
- Each file declares `namespace: chaos-testing` and scopes the selector to one service or
  infrastructure layer
- Composite experiments (`eel-*-composite.yaml`) chain multiple CRDs via `workflow` kind
- Duration defaults: 5 min for spot checks, 30 min for full rehearsals (override with `duration`)

## File inventory

| File | CRD kind | Target | Fault |
|------|----------|--------|-------|
| `eel-cert-expiry-15min.yaml` | HTTPChaos | All mTLS peers | Simulates 15-min cert TTL to test SPIRE rotation |
| `eel-debezium-pause.yaml` | PodChaos | Debezium connector pod | Pauses CDC to test outbox catch-up |
| `eel-dns-chaos.yaml` | DNSChaos | biometric-edge-svc | DNS failure forcing cached resolution |
| `eel-hsm-unavailable.yaml` | NetworkChaos | transparency-svc to HSM | Blocks PKCS#11 socket traffic |
| `eel-io-chaos-disk-full.yaml` | IOChaos | transparency-svc | Fills disk to 95% to trigger read-only mode |
| `eel-kafka-isr-shrink.yaml` | PodChaos | kafka-broker-1 | Kills one broker to test ISR rebalance |
| `eel-minio-site-loss.yaml` | NetworkChaos | All MinIO clients | Partitions MinIO to test retry + CRDT merge |
| `eel-network-loss-30pct-2s-latency.yaml` | NetworkChaos | rts-svc egress | 30% packet loss + 2 s latency to Kafka |
| `eel-pg-primary-kill-patroni-failover.yaml` | PodChaos | PostgreSQL primary pod | Tests Patroni promotion within 20 s |
| `eel-pod-kill-abis-svc.yaml` | PodChaos | abis-svc | Random pod kill to test job re-queue |
| `eel-pod-kill-ai-platform-svc.yaml` | PodChaos | ai-platform-svc | ONNX runtime pod kill |
| `eel-pod-kill-biometric-edge-svc.yaml` | PodChaos | biometric-edge-svc | Edge service restart test |
| `eel-pod-kill-boundary-svc.yaml` | PodChaos | boundary-svc | Geofence service restart |
| `eel-pod-kill-bvi-svc.yaml` | PodChaos | bvi-svc | Voter ID verification restart |
| `eel-pod-kill-bvr-svc.yaml` | PodChaos | bvr-svc | Voter registration restart |
| `eel-pod-kill-crms-svc.yaml` | PodChaos | crms-svc | Complaints system restart |
| `eel-pod-kill-ecc-svc.yaml` | PodChaos | ecc-svc | Electoral commission coordination restart |
| `eel-pod-kill-field-mobile-svc.yaml` | PodChaos | field-mobile-svc | Field officer app backend restart |
| `eel-pod-kill-governance-svc.yaml` | PodChaos | governance-svc | Governance module restart |
| `eel-pod-kill-inventory-svc.yaml` | PodChaos | inventory-svc | Materials inventory restart |
| `eel-pod-kill-portal-svc.yaml` | PodChaos | portal-svc | Public portal restart under load |
| `eel-pod-kill-queue-svc.yaml` | PodChaos | queue-svc | CV model queue restart |
| `eel-pod-kill-rts-svc.yaml` | PodChaos | rts-svc | Results transmission restart |
| `eel-pod-kill-support-admin-svc.yaml` | PodChaos | support-admin-svc | Support desk restart |
| `eel-pod-kill-training-svc.yaml` | PodChaos | training-svc | Training module restart |
| `eel-pod-kill-transparency-svc.yaml` | PodChaos | transparency-svc | Append-only log restart |
| `eel-pod-kill-vprs-svc.yaml` | PodChaos | vprs-svc | Voter print/results restart |
| `eel-sequencer-stall.yaml` | HTTPChaos | transparency-svc sequencer | Stalls sequencer API to test queue drain |
| `eel-site-partition-primary-mirror.yaml` | NetworkChaos | cross-site link | Full site partition for DR test |
| `eel-spicedb-down.yaml` | PodChaos | SpiceDB | Authz engine down to test cache fallback |
| `eel-stress-chaos-cpu-mem.yaml` | StressChaos | rts-svc / portal-svc | CPU 95% + memory 90% |
| `eel-svid-revocation.yaml` | HTTPChaos | SPIRE server | Forces SVID revocation to test rotation |
| `eel-time-chaos-plusminus-10min.yaml` | TimeChaos | biometric-edge-svc | Clock skew +/-10 min to stress HLC |
| `eel-valkey-flush.yaml` | PodChaos + exec | Valkey pods | FLUSHALL to simulate eviction surge |
| `_pod-kill-template.sh` | Bash helper | Any pod | Generic pod-kill wrapper used by CI |
| `eel-election-day-composite.yaml` | Workflow | Multi-service | Composite: scenarios 01, 04, 08, 09, 20 in sequence |
| `eel-infra-stress-composite.yaml` | Workflow | Infra layer | Composite: scenarios 03, 06, 14, 27, 28 in parallel |

## Applying an experiment

```bash
# Single experiment
kubectl apply -f chaos/mesh-experiments/eel-kafka-isr-shrink.yaml

# Check phase (should reach Running then Finished)
kubectl get podchaos -n chaos-testing eel-kafka-isr-shrink -o jsonpath='{.status.experiment.phase}'

# Composite workflow
kubectl apply -f chaos/mesh-experiments/eel-election-day-composite.yaml
kubectl get workflow -n chaos-testing -w

# Teardown
kubectl delete -f chaos/mesh-experiments/eel-election-day-composite.yaml
```

## Editing an experiment

Prefer editing `duration` and `mode` fields at the top of each file. Do not change `namespace` from
`chaos-testing` without updating the Chaos Mesh RBAC. Use `selector.labelSelectors` to scope to a
specific replica if needed.

## Adding a new experiment

1. Copy the closest existing YAML as a template.
2. Name it `eel-<fault-type>-<target>.yaml`.
3. Add a row to the table above and to `chaos/scenarios/README.md`.
4. Test in staging before committing.

## Dependencies

- Chaos Mesh >= 2.6 with `chaos-testing` namespace created
- `kubectl` context pointed at staging or pre-prod cluster
- SLO Grafana dashboard accessible for pass/fail observation

## Related docs

- `chaos/scenarios/README.md` -- scenario catalogue with expected results
- `chaos/README.md` -- programme overview and cadence
