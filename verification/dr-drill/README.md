<title>DR Drill Automation</title>

# DR Drill Automation

Automated disaster-recovery drill for the three-site topology (primary, mirror, edge). Validates
RTO and RPO targets from ADR-0004 and ADR-0019 under simulated site failures.

## Purpose

WP-6.2. Produces signed evidence that the platform recovers within RTO/RPO bounds after full site
loss. Required for gate G6.

## RTO/RPO targets

| Scenario | RTO target | RPO target |
|----------|-----------|-----------|
| Primary site loss (mirror takes over) | 120 s | 0 (sync replication) |
| Mirror site loss (primary continues) | 30 s | 0 |
| Both sites lose contact with edge | 0 s (edge operates offline) | sync on reconnect |
| PostgreSQL primary kill (Patroni) | 20 s | 0 (synchronous commit) |
| Full site network partition | 180 s | 0 |

## Modes

### dry-mode

Validates DR runbook steps, checks Patroni config, verifies Velero backup existence, and confirms
MirrorMaker2 replication lag is below threshold. No failover performed.

```bash
just dr-drill dry
```

### live-mode

Executes a controlled failover: kills primary site, measures time to mirror promotion, verifies
data integrity, then fails back.

```bash
just dr-drill live ENV=preprod SITE=primary
```

Warning: live-mode causes a real failover in the target environment. Use preprod only.

## Drill sequence

1. Pre-drill snapshot: `velero backup create dr-pre-drill --wait`
2. Confirm replication lag below 5 s on MirrorMaker2
3. Inject site partition: apply `chaos/mesh-experiments/eel-site-partition-primary-mirror.yaml`
4. Measure time to Patroni promotion (logged to `dr-results.json`)
5. Verify mirror site serves all traffic: run `./smoke-test.sh --site mirror`
6. Confirm transparency log continuity: no gap in sequence numbers
7. Restore connectivity: delete chaos experiment
8. Verify primary rejoins as follower within 300 s
9. Fail-back and restore snapshot if needed

## Expected artefacts

Written to `verification/reports/dr-drill-<date>/`:

- `dr-results.json` -- RTO/RPO measurements per scenario
- `patroni-timeline.json` -- Patroni cluster timeline events
- `mirrormaker2-lag-metrics.json` -- Kafka replication lag during drill
- `transparency-continuity.json` -- sequence number gap analysis
- `smoke-test-report.json` -- service health on mirror after failover
- `gate-g6-dr.sig` -- signed evidence for G6

## Pass criteria

- All RTO targets met (measured wall-clock from fault injection to healthy)
- RPO = 0 verified by row count and sequence number comparison
- No split-brain detected (Patroni `paused: false` on exactly one primary)
- Transparency log has no sequence gaps

## Dependencies

- Three-site Kubernetes topology deployed (primary, mirror, at least one edge)
- Patroni >= 3.2 with synchronous replication enabled
- Velero backup job completed within last 24 h
- MirrorMaker2 replication lag below 5 s before drill

## Related docs

- `chaos/mesh-experiments/eel-site-partition-primary-mirror.yaml`
- `verification/chaos-runner/README.md`
- `docs/runbooks/patroni-failover.md`
- ADR-0004 (multi-site topology)
