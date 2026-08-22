# Election-day mode: behaviour reference

## Purpose

Reference for what election-day mode does to service behaviour. This is not a procedure; for the activation and deactivation procedure, see `election-day-mode-toggle.md`. Election-day mode is a global flag set via `governance-svc` that tightens operational posture during the election window.

## What changes in election-day mode

### Write-path hardening

- **Idempotency is enforced strictly.** Every write endpoint requires an `Idempotency-Key`; duplicate submissions with the same key return the cached response instead of re-executing (EC-DAT-01/02).
- **Degradation modes are locked.** Services cannot be flipped to a weaker degradation mode without dual-officer sign-off (EC-OPS-09). The flag is reversible but every change is logged.
- **Load shedding is priority-aware.** The priority load shedder (CoDel) drops low-priority traffic before election-critical writes (EC-SCL-04).

### Read-path behaviour

- **Portal serves signed snapshots.** The public portal serves only signed publication tarballs with Merkle proofs; cache keys include the signature version (EC-SEC-12).
- **Verifier traffic is rate-limited.** STH and proof endpoints apply jitter and ETag 304 to absorb thundering herds (EC-SCL-02).

### Kit and mobile behaviour

- **Time fence is enforced.** Kits refuse to open outside the gazetted polling window unless a signed extension is applied (EC-TIM-07).
- **Attestation is strict.** Kits and mobile devices must pass attestation; failures route to the USSD/SMS fallback path (EC-IDN-04).
- **Mode switch is acked.** Kit mode changes require a signed bundle acknowledgement; unacked kits are listed in the ECC (EC-KIT-04).

### Monitoring

- **SLO burn alerts are elevated.** Election-critical services (M01, M02, M03, M05, M06, M08, M09, M10, M27, M49, M50, M51, M63) alert on tighter burn rates.
- **ECC telemetry is live.** Kit telemetry snapshots and S44A dual-control events stream to the ECC in near-real time.

## What does NOT change

- The transparency log continues to accept leaves at the same cadence; election-day mode does not pause the log.
- The Kafka outbox and Debezium pipeline are unchanged; events flow normally.
- No service is taken offline; election-day mode is a posture change, not a feature toggle.

## Verification

Confirm the flag is active and propagated:

```bash
curl -sf http://governance-svc.eel-system.svc.cluster.local:8080/admin/election-mode
# must return "mode":"election-day"
for svc in bvr-svc bvi-svc rts-svc abis-svc; do
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:9090/metrics \
    | grep 'eel_election_day_mode'
done
# each service must report the flag value 1
```

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) for emergency mode changes
- Security Officer (contact via encrypted comms) if the flag is tampered with

## Estimated duration

Not applicable; this is a reference document.

## Chaos-drill reference

The flag is exercised implicitly by any chaos scenario run in election-day mode context. The toggle procedure is exercised in tabletop drills only. See `docs/runbooks/election-day-mode-toggle.md`.
