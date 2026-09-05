<title>Chaos Scenario Catalogue</title>

# Chaos Scenario Catalogue

118 scenarios covering network, compute, storage, identity, application faults, integration failures, and insider risks. Each scenario maps to one or more SLO thresholds from ADR-0009 and a service degradation mode from ADR-0019.

## How to read this table

- **Tool/CRD kind**: the Chaos Mesh CRD or script used to inject the fault
- **Expected result**: observable system behaviour that constitutes a passing run
- Scenarios marked **COMPOSITE** appear in `mesh-experiments/eel-election-day-composite.yaml` or
  `eel-infra-stress-composite.yaml`

## Scenario catalogue (68 scenarios — 34 original + 34 extended)

| # | Scenario name | Target service(s) | Tool / CRD kind | Expected result | Audit chain | ADR refs |
|---|---------------|-------------------|-----------------|-----------------|------------|---------|
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

## Technical Explanation — Role / Purpose / Functionality / Configuration

This folder defines chaos scenarios for M5 audit validation (ADR-0022, ADR-0023, ADR-0009, ADR-0019). Each `.yaml` applies a Chaos Mesh `v1alpha1` fault (PodChaos, NetworkChaos, IOChaos, StressChaos, TimeChaos, HTTPChaos) to `chaos-testing` namespace, with an `eel.io/audit` label containing `auditId` (UUID7), `proofHash` (SHA-256), `correlation_key`, and M3 device linkage. The folder's `README.md` maps scenarios to SLO thresholds; `TECHNICAL_EXPLANATION.md` documents role/purpose/functionality/config; new scenarios 35–68 cover all 21 services with composite stress, identity storms, storage loss, HLC drift, sequence stalls, audit replay, canary storms, auth cascades, batch overflows, endpoint checks, and election-day composite.

Relevant references:
- `chaos/mesh-experiments/README.md`
- `chaos/scenarios/TECHNICAL_EXPLANATION.md`
- `platform/argocd/` (deployment of chaos test envs)
- `docs/runbooks/chaos-recovery/`

*Extended to 68 scenarios — all annotated with M5 audit chain references.*

## Related docs

- `chaos/mesh-experiments/README.md` -- YAML file reference
- `verification/chaos-runner/README.md` -- automated orchestration
- `docs/runbooks/` -- per-service degradation runbooks

### Extended scenarios (35-68) — M5 audit chain annotated
| # | Scenario | Service | Tool | Expected | M5 audit | ADR |
|---|----------|---------|------|----------|---------|-----|
| 35 | DB primary kill + replication lag | rts-svc | PodChaos | Patroni failover <20s; no message loss past sync commit | auditId+hash+key | 0009,0019 |
| 36 | Storage site loss partial read | transparency-svc | IOChaos | Read-only degraded; outbox queued; data integrity preserved | auditId+hash | 0006,0009 |
| 37 | Sequence stall on audit log | transparency-svc | PodChaos | Alarm <5s; backlog queued; ordered commit on resume | auditId+hash+key | 0022 |
| 38 | HLC clock drift extreme | biometric-edge-svc | TimeChaos | Monotonic UUIDv7 preserved; skew alarm; no rejection | auditId+hash | 0004,0006 |
| 39 | Network partition dual-site | governance-svc | NetworkChaos | Mirror takes over; no split-brain; quorum restored | auditId+hash+correlation | 0019 |
| 40 | Kafka ISR shrink to 2 brokers | queue-svc | PodChaos | ISR rebalance <60s; consumer lag <5000 | auditId+hash | 0009 |
| 41 | Certificate revocation storm | portal-svc | HTTPChaos | SPIRE rotates; zero drop during rotation | auditId+hash+sig | 0019 |
| 42 | MinIO site loss 3 replicas | portal-svc | IOChaos | Ceph rebalance <10min; no object loss | auditId+hash | 0009 |
| 43 | Workforce shift overload | workforce-svc | StressChaos | HPA fires; p99 <2s; no dropout | auditId+hash | 0010 |
| 44 | Voter lookup storm 100k/sec | vprs-svc | StressChaos | Sustains burst; p99 <500ms | auditId+hash | 0009 |
| 45 | Boundary geo-fence failure | boundary-svc | NetworkChaos | Unauthorized blocks; legit unaffected; Hubble logs | auditId+hash+key | 0019 |
| 46 | AI inference timeout 2s | ai-platform-svc | HTTPChaos | Circuit breaker opens; graceful timeout; no cascade | auditId+hash | 0019 |
| 47 | Attendance batch overflow | workforce-svc | StressChaos | Queue handles overflow; no data loss | auditId+hash | 0009 |
| 48 | Audit replay storm | transparency-svc | HTTPChaos | Replay handled; dedup by idempotency; no dupes | auditId+hash | 0023 |
| 49 | Field mobile sync disconnect loop | field-mobile-svc | NetworkChaos | Queue persists; dedup on reconnect; no duplicates | auditId+hash+key | 0022 |
| 50 | Canary token trigger storm | field-mobile-svc | HTTPChaos | Alerts fire; freeze mode; token verification | auditId+hash+correlation | 0010 |
| 51 | Mobile BFF auth cascade | field-mobile-svc | NetworkChaos | Token cache absorbs delay; retry succeeds | auditId+hash+sig | 0019 |
| 52 | Mobile verify biometric stall | field-mobile-verify | StressChaos | Bio module stalls; manual adjudication path | auditId+hash | 0006 |
| 53 | Governance vote batch overflow | governance-svc | StressChaos | Batch handles; no privilege escalation | auditId+hash | 0019 |
| 54 | Asset return seal failure | inventory-svc | IOChaos | Seal verification fails; incident recorded; chain preserved | auditId+hash | 0022 |
| 55 | Credential revocation storm | bvr-svc | HTTPChaos | Revocation propagates; no stale cert usage | auditId+hash+sig | 0009 |
| 56 | CRMS coalition split storm | crms-svc | StressChaos | Coalition split handled; party eligibility preserved | auditId+hash | 0009 |
| 57 | ECC HSM timeout cascade | ecc-svc | IOChaos | Signing queues; no unsigned entries committed | auditId+hash+sig | 0023 |
| 58 | ABIS gallery shard mismatch | abis-svc | PodChaos | Shard mismatch handled; manual review; no false match | auditId+hash | 0004,0006 |
| 59 | Edge biometric split-brain | biometric-edge-svc | NetworkChaos | Split-brain handled; CRDT merge; no duplicate | auditId+hash | 0019 |
| 60 | BVI identification storm | bvi-svc | StressChaos | ID storm handled; identification preserved; no crash | auditId+hash | 0010 |
| 61 | BVR enrolment overflow | bvr-svc | StressChaos | Enrol handled; no overflow error to clients | auditId+hash | 0009 |
| 62 | Portal tally overload | portal-svc | HTTPChaos | Tally overloaded; rate limit; upstream unaffected | auditId+hash | 0019 |
| 63 | Queue pipeline overflow | queue-svc | StressChaos | Queue handled; CV model reloads; no job lost | auditId+hash | 0010 |
| 64 | RTS tally storm | rts-svc | StressChaos | Tally sustains storm; p99 <2s; lag recovers | auditId+hash | 0009 |
| 65 | Support budget storm | support-admin-svc | StressChaos | Budget handled; fiscal integrity preserved | auditId+hash | 0019 |
| 66 | Transparency audit replay | transparency-svc | HTTPChaos | Replay handled; no duplicate audit entries | auditId+hash | 0023 |
| 67 | Workforce attendance batch | workforce-svc | HTTPChaos | Batch handled; attendance integrity preserved | auditId+hash | 0019 |
| 68 | Election-day composite stress | all | StressChaos + NetworkChaos + IOChaos | All services survive composite; M5 audit chain complete; no corruption | auditId+hash+sig+correlation | 0009,0019,0022,0023 |

### Subfolder expansions (34 integration-failure + 16 insider-risk = 50 new)
- `integration-failure/`: 34 adapter outage scenarios covering all 50+ external integrations (telco-sms, telco-ussd, SMTP, IPFS, printer, HSM-PKCS11, WhatsApp, M-Pesa, IPRS, NHIF-SHA, KBC-SFTP, NSSF, NRB, KPS-KDF-UNHCR, ODPC-breach, NIST-beacon, IFMIS, Datacite, eCourt, NLLB, LiteLLM, CBK, card-manufacturer, KEBS-warranty, MCK, Pantone, PPdt, IVR, airline-cargo, printer-MES, M-Pesa-standby, satellite, comprehensive-all). Tests that outbox drains, audit chain preserves, and no data is lost when any external adapter goes offline.
- `insider-risk/`: 16 adversarial/privileged-abuse scenarios (adapter data manipulation, audit log deletion, privilege escalation, duplicate event injection, supply-chain adapter tamper, credential theft, vote manipulation, HSM key extraction, data exfiltration, session hijack, gallery poisoning, outbox tampering, observer tampering, audit replay attack, M-Pesa double-spend, whitelist bypass). Tests that M5 audit chain + HSM + falco + tetragon + deny.toml detect and block insider actions.
- All 50 annotated with M5 audit chain: `auditId` (UUID7), `proofHash` (SHA-256), `correlation_key`, M3 device linkage, ADR-0022/0023 references.

### Technical Explanation — Role / Purpose / Functionality / Configuration

This folder defines chaos scenarios for M5 audit validation (ADR-0022, ADR-0023, ADR-0009, ADR-0019). Each `.yaml` applies a Chaos Mesh `v1alpha1` fault to `chaos-testing` namespace, with M5 audit chain labels (`auditId` UUID7, `proofHash` SHA-256, `correlation_key`, `M3_device_id`). `README.md` maps scenarios to SLO thresholds; `TECHNICAL_EXPLANATION.md` documents role/purpose/functionality/config; `TECHNICAL_EXPLANATION_INSIDER_INTEGRATION.md` covers integration failure + insider risk modeling in detail.

### Related docs
- `chaos/scenarios/TECHNICAL_EXPLANATION.md`
- `chaos/scenarios/TECHNICAL_EXPLANATION_INSIDER_INTEGRATION.md`
- `chaos/scenarios/integration-failure/`
- `chaos/scenarios/insider-risk/`
- `integrations/` — adapter/double/contract-tests structure for all 50+ integrations
- `detection/` — falco-rules, sigma-rules, tetragon-policies, wazuh-config for detection
- `docs/runbooks/chaos-recovery/`
