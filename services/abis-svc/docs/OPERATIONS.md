# abis-svc Operations Runbook

Each section is a self-contained runbook for a specific failure or maintenance scenario.

## Index

- [Enrolment Queue Overflow](#enrolment-queue-overflow)
- [Adjudication SLA Breach](#adjudication-sla-breach)
- [Vendor SDK Swap](#vendor-sdk-swap)
- [Shard Rebalance](#shard-rebalance)
- [Gallery Snapshot](#gallery-snapshot)
- [Backup and Restore](#backup-and-restore)
- [DR Handoff](#dr-handoff)

---

## Enrolment Queue Overflow

Alert: `AbisEnrolmentBacklog` (queue > 10k for 10 min).

1. Check Kafka consumer lag: `kubectl exec -n kafka kafka-0 -- kafka-consumer-groups.sh --bootstrap-server localhost:9092 --describe --group abis-matcher`.
2. Scale up matcher: `kubectl scale deploy abis-matcher -n abis --replicas=<current+4>`.
3. If lag is in PG outbox: check `SELECT count(*) FROM outbox WHERE processed_at IS NULL;`.
4. If PG connections are exhausted: restart PgBouncer sidecar.
5. After queue drains, scale matcher back to HPA-managed baseline.
6. Post incident to `#eel-ops`; update ADR-0018 throughput baseline if breach recurs.

---

## Adjudication SLA Breach

Alert: `AbisAdjudicationSlaBreach` (Tier1 > 4h, 5 open cases).

1. Check case count: `SELECT tier, priority, count(*) FROM adjudication_cases WHERE status='open' AND sla_deadline_at < now() + interval '2h' GROUP BY 1,2;`.
2. Identify unassigned cases: `SELECT id, created_at, sla_deadline_at FROM adjudication_cases WHERE assigned_to IS NULL AND status='open' LIMIT 20;`.
3. Manually assign to on-call investigator via Django console (`/admin/adjudication/cases/`) or API: `PATCH /api/v1/abis/adjudication/{id} {"assigned_to":"<investigator_id>"}`.
4. If ElectionDayHold cases are queued behind Standard: verify priority queue ordering in `abis-matcher` logs (`grep "priority" logs`).
5. Scale adjudicator worker if throughput is low: `kubectl scale deploy abis-adjudicator -n abis --replicas=<N>`.

---

## Vendor SDK Swap

Alert: `AbisVendorRouterAllUnhealthy`.

1. Identify failing vendor: check `abis-matcher` logs for `vendor_router: all_unhealthy`.
2. Verify `abis-vendor` pod health: `kubectl get pods -n abis -l app.kubernetes.io/name=abis-vendor`.
3. If vendor container is crashlooping: `kubectl logs -n abis deploy/abis-vendor --previous`.
4. To promote secondary vendor: set env var `EEL_ABIS_VENDOR_PRIMARY=<secondary_url>` and rollout: `kubectl rollout restart deploy/abis-vendor -n abis`.
5. Swap SDK: update `py/abis-vendor/src/abis_vendor/adapter.py`, bump image tag in `deploy/service.yaml`, trigger CD pipeline.
6. Validate via `POST /api/v1/abis/match/11` with a synthetic probe; expect 200.

---

## Shard Rebalance

After adding shard capacity or recovering from `eel-abis-shard-loss` chaos.

1. Verify current shard distribution: `kubectl get pods -n abis -l app.kubernetes.io/name=abis-matcher -o wide`.
2. Check HNSW index size per pod via Grafana "HNSW Index Size" panel.
3. Trigger rebalance job: `kubectl create job --from=cronjob/abis-shard-rebalance eel-shard-rebalance-manual -n abis`.
4. Monitor Grafana "Shard Rebalance Activity" panel until complete.
5. Run recall@10 check: `GET /api/v1/abis/reports?type=recall` and verify >= 0.95.

---

## Gallery Snapshot

Snapshot the HNSW gallery to MinIO for cold-start recovery.

1. Trigger snapshot job: `kubectl create job --from=cronjob/abis-gallery-snapshot eel-gallery-snapshot-manual -n abis`.
2. Verify output in MinIO: `mc ls eel-internal/abis-gallery-snapshots/`.
3. Validate snapshot integrity: `GET /api/v1/abis/reports?type=snapshot-integrity`.
4. Retain snapshots for 30 days; Kopia policy handles rotation.

---

## Backup and Restore

### Backup

PostgreSQL backups run via `pgbackrest` (continuous WAL archiving to MinIO). Daily base backup at 02:00 UTC.

1. Verify backup status: `pgbackrest --stanza=abis info`.
2. Check WAL archive lag: `psql -c "SELECT pg_current_wal_lsn() - last_archived_lsn FROM pg_stat_archiver;"`.

### Restore

1. Stop `abis-api` and `abis-matcher` deployments: `kubectl scale deploy abis-api abis-matcher -n abis --replicas=0`.
2. Run restore: `pgbackrest --stanza=abis --delta restore`.
3. Promote Patroni standby if primary is lost: `patronictl -c /etc/patroni/patroni.yaml failover abis --master <old-primary> --candidate <standby> --force`.
4. Restart deployments: `kubectl scale deploy abis-api abis-matcher -n abis --replicas=3`.
5. Verify outbox resumed: check Kafka consumer lag returns to zero.

---

## DR Handoff

Full site failover from primary to mirror.

1. Confirm primary is unreachable: ping + HTTP probe fails.
2. Promote mirror Patroni cluster: `patronictl -c /etc/patroni/patroni.yaml failover abis-mirror --force`.
3. Update Cilium BGP policy to route `abis.eel.internal` to mirror.
4. Scale up mirror deployments: `kubectl scale deploy abis-api abis-matcher abis-vendor -n abis --replicas=<production_target>`.
5. Verify MirrorMaker2 lag is zero or replay from MinIO snapshot if lag > 5 min.
6. Run health check: `GET https://abis.eel.internal/healthz` from all edge sites.
7. Open incident in `#eel-ops`; engage on-call SRE for root cause.
