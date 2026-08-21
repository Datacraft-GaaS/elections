# Operations Guide

This is a high-level index. Detailed runbooks live in `docs/runbooks/` and operational tooling in `operations/`.

## Runbook Index

| Runbook | Scenario |
|---|---|
| `docs/runbooks/backup-restore-drill.md` | Scheduled backup restore verification |
| `docs/runbooks/break-glass.md` | Emergency operator access procedure |
| `docs/runbooks/chaos-monthly-drill.md` | Monthly chaos experiment schedule |
| `docs/runbooks/consumer-lag-response.md` | Kafka consumer lag remediation |
| `docs/runbooks/degradation-per-service.md` | Per-service degradation modes and recovery |
| `docs/runbooks/dr-failback.md` | Failback to primary after DR activation |
| `docs/runbooks/dr-failover-bvr-svc.md` | BVR service failover procedure |
| `docs/runbooks/dr-failover-primary-to-mirror.md` | Full site failover |
| `docs/runbooks/dr-failover-rts-svc.md` | RTS service failover (election-critical) |
| `docs/runbooks/edge-region-loss.md` | Edge cluster loss and reconnection |
| `docs/runbooks/election-day-mode-toggle.md` | Enabling/disabling election-day mode |
| `docs/runbooks/election-day-mode.md` | Election-day operations reference |
| `docs/runbooks/event-replay.md` | Kafka event replay and consumer reset |
| `docs/runbooks/hsm-key-ceremony.md` | HSM key ceremony procedure |
| `docs/runbooks/hsm-loss.md` | HSM failure and recovery |
| `docs/runbooks/incident-response-p0.md` | P0 incident response (election-day) |
| `docs/runbooks/incident-response.md` | Standard incident response |
| `docs/runbooks/kafka-broker-failover.md` | Kafka broker replacement |
| `docs/runbooks/kafka-isr-shrink.md` | In-sync replica shrink response |
| `docs/runbooks/keycloak-realm-upgrade.md` | Keycloak upgrade procedure |

## Operations Tooling

| Directory | Purpose |
|---|---|
| `operations/mdm/` | MDM profile management (Android Enterprise, ABM) |
| `operations/helpdesk/` | Helpdesk integration runbooks |
| `operations/e-learning/` | LMS operations (content refresh, certificate batch) |
| `operations/warranty/` | Warranty claim procedures (KEBS, OEM) |
| `operations/dispatch/` | Field dispatch procedures |
| `operations/data-migration/` | KIEMS legacy data migration tooling |

## Monitoring and Alerting

All services export Prometheus metrics via the `eel-observability-*` libraries. Grafana dashboards are in `platform/grafana/`. Alert rules are in `platform/prometheus/alerts/`.

Key dashboards:
- **Election Day Overview**: kit health, turnout by region, form transmission rates.
- **Service SLO Burn**: per-service error budget burn rate.
- **Kafka Consumer Lag**: per-consumer-group lag with alert thresholds.
- **HSM Health**: enclave attestation freshness, key ceremony status.

## MDM

Field kits run Android Enterprise (COPE mode) managed by the MDM server at `operations/mdm/`. Kit provisioning zero-touch enrolment is triggered by `just mdm-enroll <serial>`. De-provisioning is triggered by `just mdm-wipe <serial>` and generates a custody event in inventory-svc.

## On-call rotation

On-call engineers are paged via PagerDuty when any SLO burn alert fires. The primary on-call has 5 min to acknowledge; escalation to the secondary at 10 min. P0 (election-day write path failure) escalates to the incident commander immediately. See `docs/runbooks/incident-response-p0.md`.
