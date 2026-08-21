# diaspora container seal break runbook

Module: M65 diaspora-ballot-physical-logistics. WP-4.3.

## Trigger

Seal-broken event detected at any custody transfer point (embassy, hub, Nairobi).

Alert: `eel.custody.seal_broken.v1` fired on M65; P1 delivered to ECC p99 within 5 min (FR-M65-040).

## Steps

1. ECC alert fires. Container status transitions to `quarantined` in `ballot_containers`.
2. Hub Coordinator or tally-centre officer notifies IAB observer and party agents on-site.
3. KNPS liaison notified if P0 escalation (container not at expected location).
4. IAB observer witnesses physical inspection of container contents.
5. Photograph all seals and contents; store SHA-256 photo hash in custody event.
6. If contents intact: re-seal with new seal UID from M20 inventory.
   - Re-seal event recorded in `container_custody_events` with new seal UID.
   - IAB witness record required.
7. If contents compromised: escalate to IEBC Commissioner and Commissioner page within 30 min.
8. Container status updated to `in_transit` or `received` after successful re-seal.

## EC-OPS-03: seal broken in transit at hub

P1 alert to ECC and IAB within 60 s (AC-M65-01). Container quarantined until inspected. Re-seal requires new M20 seal UID and IAB witness. All steps logged in `ballot_containers_audit`.

## EC-SEC-02: tamper detected at hub scan

Quarantine stream event raised. ECC alert fired. Container FSM transitions to `quarantined`. Cannot proceed to Nairobi without Commissioner sign-off and IAB re-witness.
