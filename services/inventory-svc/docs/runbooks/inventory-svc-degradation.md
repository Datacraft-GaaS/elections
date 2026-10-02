# inventory-svc degradation runbook

Service: inventory-svc. WP-4.3. Modules: M20, M39, M65.

## HSM fail (EC-SEC-03)

Custody and seal write endpoints fail closed. STH pauses. Ingest continues in degraded mode.

Steps:
1. Alert fires on `eel.custody.hsm_unavailable`. Page on-call SRE.
2. Check HSM cluster health via `hsm-admin status`.
3. If SoftHSM in dev: `systemctl restart softhsm2`.
4. Prod: engage Datacraft HSM ops within 15 min (SLA: P0 2h).
5. On recovery, drain custody_outbox: events awaiting HSM signature replay automatically.
6. Verify Merkle sequence continues from last checkpoint.

## RFID reader outage (EC-OPS-08)

Alert fires when manual barcode entry rate exceeds 20% of total scans.

Steps:
1. Check `rfid_reader_fail_rate` metric in Grafana (dashboard: eel-iems-20).
2. Notify field engineer for reader at affected gate (P0 SLA: 2h).
3. Manual barcode entry is the fallback; log rate above 20% as incident.
4. On reader recovery, verify RFID scan events resume; manual flag clears.

## Reconciliation failure (FR-M20-029)

Station close reconciliation not submitted within 2h triggers auto-alert to M16.

Steps:
1. Check `station_reconciliations` for unsubmitted stations.
2. Contact Presiding Officer via ECC ops channel.
3. If M08 kit offline: confirm WAL queue is draining after reconnect.
4. Escalate to Constituency Returning Officer if no response within 30 min.

## Reserve alert escalation (FR-M20-035)

Reserve pool below threshold: alert to ECC within 60s.

Steps:
1. Alert in ECC: check tier (L1 county / L2 regional / L3 air-freight).
2. L1: County Coordinator dispatches spare within 2h.
3. L2: Regional Hub Coordinator dispatches within 4h.
4. L3 air-freight emergency: 30-min activation SLA; engage the Commission Ops Director.

## Ballot printing chain: offline queue (M39)

M08 RFID scanner offline at station receipt; local event queue persists up to 48h.

Steps:
1. Confirm M08 device shows "offline mode" indicator.
2. Station cannot confirm open until receipt anchored; notify field engineer.
3. On reconnect, verify no duplicate events submitted (idempotency key check).
4. If sync fails after reconnect: check connectivity; retry with exponential backoff.

## Diaspora logistics: seal break (M65)

Seal-broken event at hub triggers P1 alert to ECC and IAB.

Steps:
1. Alert fires within 60s (AC-M65-01).
2. Container quarantined in `ballot_containers.status = 'quarantined'`.
3. IAB observer must witness re-inspection.
4. Re-seal requires new seal UID from M20 inventory and IAB witness record.
5. Update `container_custody_events` with re-seal event; resume routing.
