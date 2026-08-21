# Election-day mode toggle: dual-officer activation and reversal

## Purpose

Activate or deactivate the election-day mode flag via the `governance-svc` API. This is the procedural runbook for the toggle action. For documentation of what election-day mode does to service behaviour, see `election-day-mode.md`.

Election-day mode requires dual-officer authentication and is fully reversible. Every activation and deactivation is recorded in the transparency log.

## Prerequisites

- Two authorised officers present (`<ID_A>` and `<ID_B>`), both authenticated to the governance-svc admin scope.
- `governance-svc` admin API is reachable.
- An ITSM change ticket open as `<TICKET>` (pre-approved for scheduled activation; emergency ticket for unscheduled).
- Time synchronised (NTP offset less than 5 seconds on the ops jump host).

## Commands

### Activation

#### Step 1: confirm pre-conditions

```bash
# verify both officers are authenticated:
curl -sf http://governance-svc.eel-system.svc.cluster.local:8080/admin/session \
  -H "Authorization: Bearer <TOKEN_A>" | grep '"officer_id"'
curl -sf http://governance-svc.eel-system.svc.cluster.local:8080/admin/session \
  -H "Authorization: Bearer <TOKEN_B>" | grep '"officer_id"'
# verify current mode is "normal":
curl -sf http://governance-svc.eel-system.svc.cluster.local:8080/admin/election-mode \
  | grep '"mode"'
```

#### Step 2: activate election-day mode

```bash
curl -sf -X POST \
  http://governance-svc.eel-system.svc.cluster.local:8080/admin/election-mode \
  -H "Content-Type: application/json" \
  -d "{
    \"mode\": \"election-day\",
    \"ticket\": \"<TICKET>\",
    \"officer_a\": \"<ID_A>\",
    \"officer_b\": \"<ID_B>\",
    \"effective_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }"
# expected: {"mode":"election-day","effective_at":"<ISO8601>","status":"active"}
```

#### Step 3: verify mode is active

```bash
curl -sf http://governance-svc.eel-system.svc.cluster.local:8080/admin/election-mode
# must return "mode":"election-day"
# verify downstream services have picked up the flag:
for svc in bvr-svc bvi-svc rts-svc abis-svc; do
  echo -n "$svc election_mode: "
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:9090/metrics \
    | grep 'eel_election_day_mode'
done
```

#### Step 4: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.election-day-mode.activated.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Deactivation (reversal)

#### Step 1: confirm post-election conditions are met

```bash
# verify tally declaration is complete in rts-svc:
curl -sf http://rts-svc.eel-system.svc.cluster.local:8080/admin/declaration-status
# and that all results have been committed to tlog
```

#### Step 2: deactivate election-day mode

```bash
curl -sf -X POST \
  http://governance-svc.eel-system.svc.cluster.local:8080/admin/election-mode \
  -H "Content-Type: application/json" \
  -d "{
    \"mode\": \"normal\",
    \"ticket\": \"<TICKET>\",
    \"officer_a\": \"<ID_A>\",
    \"officer_b\": \"<ID_B>\",
    \"effective_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }"
# expected: {"mode":"normal","status":"active"}
```

#### Step 3: verify services have reverted

```bash
for svc in bvr-svc bvi-svc rts-svc abis-svc; do
  echo -n "$svc election_mode: "
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:9090/metrics \
    | grep 'eel_election_day_mode'
  # metric value must be 0
done
```

#### Step 4: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.election-day-mode.deactivated.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `governance-svc` admin API returns the expected mode after toggle.
2. All service metrics show the `eel_election_day_mode` counter matching the new mode (1 for active, 0 for normal).
3. Transparency log contains the mode-change leaf.
4. Ticket is updated with both officer IDs and tlog leaf hash.

## Rollback procedure

Deactivation is the rollback for activation (and vice versa). Both directions follow the same dual-officer procedure above. No infrastructure restart is required; the flag change propagates via the governance-svc config push mechanism within 30 seconds.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge) for emergency activation outside schedule
- Security Officer (contact via encrypted comms) if the flag is tampered with

## Estimated duration

5 minutes.

## Chaos-drill reference

This runbook is exercised in tabletop drills only. The flag itself is exercised implicitly by any chaos scenario run in election-day mode context. For mode-specific service behaviour, see `docs/runbooks/election-day-mode.md`.
