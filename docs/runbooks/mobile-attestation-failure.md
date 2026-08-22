# Mobile attestation failure at scale

## Purpose

Respond to a spike in mobile device attestation failures. Attestation (Play Integrity, App Attest, TPM, or the legacy software tier) gates access to accredited mobile apps. A spike can indicate a genuine compromise wave, a broken attestation provider, or a bad app update. The goal is to distinguish these, contain a real compromise, and restore service to legitimate devices.

## Prerequisites

- Access to the attestation service logs and metrics.
- The mobile app version floor and attestation policy documented in `platform/mobile/attestation-policy.yaml`.
- An incident ticket open as `<TICKET>`.
- Security Officer notified; attestation failures are a security-relevant signal.

## Commands

### Step 1: quantify the failure

```bash
# measure the failure rate and which tier is failing:
curl -sf http://prometheus.eel-system.svc.cluster.local:9090/api/v1/query \
  --data-urlencode 'query=sum(rate(eel_attestation_failures_total[5m])) by (tier, reason)' \
  | python3 -m json.tool
# group by reason: integrity, device binding, version floor, provider error
```

### Step 2: check the attestation provider

```bash
# if failures are concentrated in one tier, the provider may be degraded:
# check Play Integrity / App Attest / TPM provider status
# a provider outage produces a broad, uniform failure spike
```

### Step 3: check for a bad app update

```bash
# if failures correlate with a recent app release, check the version floor:
# a version floor bump that is too aggressive rejects legitimate devices
# compare the failing version distribution against the release timeline
```

### Step 4: check for a genuine compromise wave

```bash
# if failures are concentrated on specific device models or regions,
# treat it as a possible compromise wave:
# - revoke the affected device class (see kit-mass-revocation.md)
# - route affected devices to the USSD/SMS fallback path (EC-IDN-04)
# - capture evidence for the ECC
```

### Step 5: apply the appropriate mitigation

```bash
# provider outage: wait for recovery; do not lower the attestation bar
# bad update: roll back the app or adjust the version floor with a signed config push
# compromise wave: revoke and route to fallback; do not weaken attestation
# in all cases, do not disable attestation; that is a security regression
```

### Step 6: monitor recovery

```bash
watch -n 10 'curl -sf http://prometheus.eel-system.svc.cluster.local:9090/api/v1/query \
  --data-urlencode "query=sum(rate(eel_attestation_failures_total[5m]))" \
  | python3 -m json.tool'
# wait for the failure rate to return to baseline
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.mobile-attestation-failure.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"root_cause\":\"<CAUSE>\",\"devices_affected\":\"<N>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Attestation failure rate returns to baseline.
2. Legitimate devices can re-authenticate (no persistent false-fail).
3. If a compromise wave was confirmed, affected devices are revoked and routed to fallback.
4. Attestation was never disabled.
5. Transparency log contains the resolved leaf.

## Rollback procedure

If a mitigation (version floor change, config push) made things worse, reverse it with a signed config push. Do not disable attestation as a rollback; that is a security regression. If a compromise wave is confirmed, the rollback is revocation, not re-enablement.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- Site Reliability Lead (on-call via PagerDuty rotation)
- Mobile Platform Lead (contact via war room bridge) for app release issues

## Estimated duration

30 to 90 minutes depending on root cause.

## Chaos-drill reference

This runbook is exercised in tabletop drills only. It is related to `docs/runbooks/kit-mass-revocation.md` (compromise wave) and `docs/runbooks/stolen-kit-honey-mode.md` (single-device compromise).
