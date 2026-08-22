# Tamper topic response

## Purpose

Respond to a tamper event detected on a monitored Kafka topic. A tamper event indicates a signed message failed signature verification, a replay was rejected, or a seal/chassis was broken. The goal is to contain the tampered message, quarantine it, alert the ECC, and determine whether it is a genuine compromise or a false positive.

## Prerequisites

- Access to the tamper topic and the event consumer logs.
- The signing and verification policy documented in `docs/EVENT-CONVENTIONS.md`.
- An incident ticket open as `<TICKET>`.
- Security Officer notified; tamper events are a security-relevant signal.

## Commands

### Step 1: confirm the tamper event

```bash
# read the tamper topic and confirm the event is genuine:
kafka-console-consumer.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --topic eel-tamper --from-beginning --max-messages 10
# record the tamper event id and the reason (signature, replay, seal)
```

### Step 2: quarantine the tampered message

```bash
# the consumer routes tampered messages to the .tamper topic by design:
# confirm the message is quarantined and not applied to any projection
# verify the producer that emitted it is identified
```

### Step 3: page the ECC

```bash
# raise an ECC alert for the tamper event:
# include the event id, reason, producer, and affected stream
# the alert must page within 30 seconds of detection
```

### Step 4: determine the cause

```bash
# classify the tamper:
# - signature mismatch: a key was rotated or a message was modified in transit
# - replay: a counter was reused; check for a compromised producer
# - seal/chassis: a physical tamper; see kit-mass-revocation.md
# check the producer's key rotation timeline and the message's signing key
```

### Step 5: apply the appropriate mitigation

```bash
# signature mismatch from key rotation: confirm the rotation is complete and re-verify
# genuine compromise: revoke the affected key/device and route to fallback
# false positive: document the cause and close the ticket
# in all cases, do not apply the tampered message to any projection
```

### Step 6: monitor for further tamper events

```bash
watch -n 10 'kafka-console-consumer.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --topic eel-tamper --from-beginning --max-messages 1'
# wait for the tamper rate to return to baseline
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.tamper-topic.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"tamper_event\":\"<EVENT_ID>\",\"root_cause\":\"<CAUSE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. The tampered message is quarantined and not applied to any projection.
2. The ECC alert fired within 30 seconds.
3. The root cause is classified and documented.
4. No further tamper events from the same source.
5. Transparency log contains the resolved leaf.

## Rollback procedure

A tampered message is never applied; there is nothing to roll back. If a false positive was caused by a key rotation, complete the rotation and re-verify the affected stream. If a genuine compromise is confirmed, the response is revocation, not re-enablement.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- Site Reliability Lead (on-call via PagerDuty rotation)
- Kafka/Streaming Lead (on-call via PagerDuty rotation)

## Estimated duration

15 to 45 minutes depending on root cause.

## Chaos-drill reference

This runbook is exercised by `chaos/scenarios/eel-tamper-injection.yaml` (forged signed message). Related: `docs/runbooks/kit-mass-revocation.md` (compromise wave) and `docs/runbooks/witness-divergence-response.md` (log split-view).
