# Runbooks index

All runbooks follow a standard structure: Purpose, Prerequisites, Commands, Verification Steps, Rollback Procedure, Escalation Contacts, Estimated Duration, Chaos-Drill Reference.

## All runbooks

| Runbook | File | Trigger condition | Duration | Chaos drill |
|---------|------|-------------------|----------|-------------|
| Break-glass | [break-glass.md](break-glass.md) | Credential loss, insider threat, emergency root access needed | 45-90 min | eel-hsm-unavailable.yaml, eel-spicedb-down.yaml |
| Keycloak realm upgrade | [keycloak-realm-upgrade.md](keycloak-realm-upgrade.md) | Realm JSON schema change, blue-green promotion | 30-60 min | eel-pod-kill-boundary-svc.yaml |
| SpiceDB schema migration | [spicedb-schema-migration.md](spicedb-schema-migration.md) | AuthZ schema change (additive or breaking) | 20-40 min | eel-spicedb-down.yaml |
| Incident response (general) | [incident-response.md](incident-response.md) | Any production incident P1 or below | varies | N/A |
| P0 incident response | [incident-response-p0.md](incident-response-p0.md) | Data loss, tally corruption, tlog divergence, HSM compromise, platform-wide write outage | 15-30 min to mitigate | eel-stress-chaos-cpu-mem.yaml |
| Election-day mode (behaviour) | [election-day-mode.md](election-day-mode.md) | Reference for what election-day mode does | N/A | N/A |
| Election-day mode toggle | [election-day-mode-toggle.md](election-day-mode-toggle.md) | Scheduled or emergency activation/deactivation of election-day flag | 5 min | tabletop only |
| DR failover: primary to mirror | [dr-failover-primary-to-mirror.md](dr-failover-primary-to-mirror.md) | Primary site unrecoverable; failover to mirror needed | 10-15 min | eel-site-partition-primary-mirror.yaml |
| DR failback: mirror to primary | [dr-failback.md](dr-failback.md) | Primary site recovered; return traffic from mirror | 20-40 min | eel-site-partition-primary-mirror.yaml |
| DR failover: bvr-svc | [dr-failover-bvr-svc.md](dr-failover-bvr-svc.md) | bvr-svc primary instance lost | varies | N/A |
| DR failover: rts-svc | [dr-failover-rts-svc.md](dr-failover-rts-svc.md) | rts-svc primary instance lost | varies | N/A |
| PG cluster failover | [pg-cluster-failover.md](pg-cluster-failover.md) | Planned Patroni switchover | 5-10 min | N/A |
| PG primary loss | [pg-primary-loss.md](pg-primary-loss.md) | Unplanned PostgreSQL primary failure; Patroni promotion | 2-5 min | eel-pg-primary-kill-patroni-failover.yaml |
| pgBackRest PITR | [pgbackrest-pitr.md](pgbackrest-pitr.md) | Data corruption requiring point-in-time restore; quarterly drill | 30-60 min | eel-pg-primary-kill-patroni-failover.yaml |
| Kafka ISR shrink | [kafka-isr-shrink.md](kafka-isr-shrink.md) | Under-replicated Kafka partitions; ISR below quorum | 15-45 min | eel-kafka-isr-shrink.yaml |
| Kafka broker failover | [kafka-broker-failover.md](kafka-broker-failover.md) | Kafka broker node failure | varies | N/A |
| Consumer lag response | [consumer-lag-response.md](consumer-lag-response.md) | Kafka consumer group lag exceeds threshold | varies | N/A |
| Edge region loss | [edge-region-loss.md](edge-region-loss.md) | Edge region unreachable; anycast reroute needed | 5-15 min | eel-network-loss-30pct-2s-latency.yaml, eel-dns-chaos.yaml |
| HSM loss | [hsm-loss.md](hsm-loss.md) | HSM offline; sequencer degradation and backup HSM activation | 10-20 min | eel-hsm-unavailable.yaml, eel-sequencer-stall.yaml |
| HSM key ceremony | [hsm-key-ceremony.md](hsm-key-ceremony.md) | Scheduled key generation or rotation ceremony | 2-4 hr | N/A |
| Kit mass revocation | [kit-mass-revocation.md](kit-mass-revocation.md) | Kit class compromised; SVID and CRL mass revocation needed | 10-20 min | eel-svid-revocation.yaml, eel-cert-expiry-15min.yaml |
| Stolen kit honey mode | [stolen-kit-honey-mode.md](stolen-kit-honey-mode.md) | Kit reported stolen; evidence capture needed | 5-10 min | tabletop only |
| SPIRE SVID mass rotation | [spire-svid-mass-rotation.md](spire-svid-mass-rotation.md) | Scheduled key rotation; post-incident re-keying; upstream CA change | 20-40 min | eel-cert-expiry-15min.yaml, eel-svid-revocation.yaml |
| Witness divergence response | [witness-divergence-response.md](witness-divergence-response.md) | Transparency log witnesses disagree on tree head | 15-60 min | eel-site-partition-primary-mirror.yaml |
| RTS tally amendment | [rts-tally-amendment.md](rts-tally-amendment.md) | Tally error found post-certification; compensating event needed | 20-40 min | tabletop only |
| Velero restore | [velero-restore.md](velero-restore.md) | Namespace-level resource loss; restore from Velero backup | 15-30 min | eel-minio-site-loss.yaml |
| Backup restore drill | [backup-restore-drill.md](backup-restore-drill.md) | Quarterly backup integrity drill | 45-90 min | N/A |
| Chaos monthly drill | [chaos-monthly-drill.md](chaos-monthly-drill.md) | Monthly chaos engineering exercise | 2-4 hr | all |
| Load test weekly drill | [loadtest-weekly-drill.md](loadtest-weekly-drill.md) | Weekly load test execution | 1-2 hr | N/A |
| Mobile attestation failure | [mobile-attestation-failure.md](mobile-attestation-failure.md) | Mobile device attestation failures at scale | varies | N/A |
| PII purge job | [pii-purge-job.md](pii-purge-job.md) | Scheduled or mandated PII deletion | 30-60 min | N/A |
| Replay projection rebuild | [replay-projection-rebuild.md](replay-projection-rebuild.md) | Read-model projection corrupted; rebuild from event log | varies | N/A |
| Event replay | [event-replay.md](event-replay.md) | Manual event replay from outbox or Kafka | varies | N/A |
| Schema migration | [schema-migration.md](schema-migration.md) | Database schema migration (non-SpiceDB) | varies | N/A |
| Tamper topic response | [tamper-topic-response.md](tamper-topic-response.md) | Tamper event detected on a monitored Kafka topic | varies | N/A |
| Transparency log witness cross-check | [transparency-log-witness-cross-check.md](transparency-log-witness-cross-check.md) | Routine or triggered witness consistency check | 15-30 min | N/A |
| Per-service degradation modes | [degradation-per-service.md](degradation-per-service.md) | Any service needing controlled degradation or read-only mode | 5 min per service | eel-pod-kill-*.yaml |

## Quick reference by trigger

| Trigger | Runbook |
|---------|---------|
| Primary site down | dr-failover-primary-to-mirror.md |
| PostgreSQL primary failure | pg-primary-loss.md |
| Kafka partitions under-replicated | kafka-isr-shrink.md |
| HSM offline | hsm-loss.md |
| Kit stolen | stolen-kit-honey-mode.md |
| Kit class compromised | kit-mass-revocation.md |
| Tlog witnesses disagree | witness-divergence-response.md |
| Tally error post-certification | rts-tally-amendment.md |
| SVID rotation needed | spire-svid-mass-rotation.md |
| Namespace resource loss | velero-restore.md |
| Data corruption requiring restore | pgbackrest-pitr.md |
| Edge region unreachable | edge-region-loss.md |
| P0 declared | incident-response-p0.md |
| Election day starting | election-day-mode-toggle.md |
| Emergency root access needed | break-glass.md |
| Service needs degradation | degradation-per-service.md |
