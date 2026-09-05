# Playbook 02: Cross-System Correlation Verification (M5 Drill Phase 5)

> Verify that all 4 detection systems (falco, wazuh, audit_service, tetragon) correlate correctly on real and simulated events. Tests the full audit chain: event → falco ALERT → wazuh ALERT → audit_event (with proofHash + Ed25519 signature) → audit_shadow updated → tetragon event.

## Purpose

M5 48h drill — Phase 5 correlation test. Ensures no false positives on normal flows and 100% correlation on canary triggers.

## Correlation Chain (Expected for Every Event)

```
1. Event triggers
   (canary access / audit tamper / biometric replay / HSM access / supply chain / MDM violation)

2. Falco ALERT emitted (within 60s of trigger)
   → Rule: eel_fingerprints.yaml / eel_extended.yaml / eel_hardware_integrity.yaml / eel_network_ddos.yaml
   → Output: node, user, command, correlation_key, M3_device_id

3. Audit_service event created (within 120s of falco ALERT)
   → eventType: matches the event
   → auditId: UUIDv7
   → proofHash: SHA-256 of payload (RFC 8785 canonical JSON)
   → Ed25519 signature: EelSigningKey over proofHash
   → correlation_key: matches falco correlation_key
   → actorId, actorRole, stationId, whenUtc

4. Wazuh ALERT emitted (within 60s of audit_service event)
   → decoded_as: matches decoder (falco-decoder / eel-audit-decoder / etc.)
   → rule id: matches wazuh rule ID (100110-101013)
   → level: 5/7/10/14
   → correlationKey: matches correlation_key from step 2

5. Tetragon event emitted (within 30s of trigger)
   → process_krn: kernel process name
   → process_path: binary path
   → namespace: host/container/kubernetes

6. Audit_shadow updated
   → Encrypted replay log: audit entry appended with Ed25519 signature
   → M3_device_id: device identifier

7. SRE pager (EMERGENCY only)
   → Triggered by: HSM key access, supply chain EMERGENCY, root remount
   → Within 30s of falco EMERGENCY
```

## Verification Steps

### Step 1 — Audit Chain Verification (for each event type)

```bash
# For a given correlation_key, verify all 4 systems emitted within window
CORR_KEY="audit-corr-vote-001"

# 1. Falco: check events.log for correlation_key
grep "$CORR_KEY" /var/log/falco/events.log | jq '.output_fields.correlation_key'

# 2. Audit_service: check for event with matching correlation_key
curl -s "https://audit.eel.local/api/v1/events?correlation_key=$CORR_KEY" \
  -H "Authorization: Bearer $AUDIT_TOKEN" | jq '.[] | {auditId, proofHash, signature}'

# 3. Wazuh: check alerts.json for rule id
grep "$CORR_KEY" /var/ossec/logs/alerts/alerts.json | jq '.rule.description'

# 4. Tetragon: check process_krn for correlation_key
grep "$CORR_KEY" /var/log/tetragon/tetragon.log | jq '.process_krn'

# 5. Audit_shadow: verify entry appended
curl -s "https://audit.eel.local/api/v1/shadow?auditId=$AUDIT_ID" \
  -H "Authorization: Bearer $AUDIT_TOKEN" | jq '.proofHash, .signature'
```

### Step 2 — False Positive Check (normal enrollment flow)

```bash
# Start a normal enrollment session and verify NO false positives
# (normal enrollment should NOT trigger canary alerts)

# 1. Run normal enrollment
cd /data/app/com.eel.bvr
./enroll_test.sh --person-id=P001 --station=ST001 --actor=operator001

# 2. Check falco events
grep -E "(canary|audit-corr|bio-corr|crypto-corr|sc-corr|mdm-corr)" \
  /var/log/falco/events.log

# Expected: 0 false positives
# If false positive found: update falco rule filter / wazuh rule filter
```

### Step 3 — Canary Token Correlation (all 17 tokens)

```bash
# For each correlation_key in correlation_routing.md, verify full chain
for KEY in $(cat ../canary-tokens/correlation_routing.md | grep "correlation_key:" | awk '{print $2}'); do
  echo "=== Verifying $KEY ==="

  FALCO=$(grep "$KEY" /var/log/falco/events.log | wc -l)
  AUDIT=$(curl -s "https://audit.eel.local/api/v1/events?correlation_key=$KEY" | jq '. | length')
  WAZUH=$(grep "$KEY" /var/ossec/logs/alerts/alerts.json | wc -l)
  TETRAGON=$(grep "$KEY" /var/log/tetragon/tetragon.log | wc -l)

  echo "  falco: $FALCO | audit: $AUDIT | wazuh: $WAZUH | tetragon: $TETRAGON"

  if [ "$FALCO" -eq 0 ] || [ "$AUDIT" -eq 0 ] || [ "$WAZUH" -eq 0 ]; then
    echo "  FAIL: Missing events in one or more systems"
  else
    echo "  PASS: Full correlation confirmed"
  fi
done
```

### Step 4 — Timing Verification

```bash
# Verify all events fall within specified windows
# Window 1: event → falco ALERT ≤ 60s
# Window 2: falco ALERT → audit_service event ≤ 120s
# Window 3: event → wazuh ALERT ≤ 120s
# Window 4: event → tetragon event ≤ 30s

for KEY in audit-corr-vote-001 canary-bio-fp-001 crypto-corr-ed25519-001; do
  echo "=== Timing for $KEY ==="
  FALCO_TS=$(grep "$KEY" /var/log/falco/events.log | jq -r '.time' | head -1)
  AUDIT_TS=$(curl -s "https://audit.eel.local/api/v1/events?correlation_key=$KEY" | jq -r '.[0].whenUtc' | head -1)
  WAZUH_TS=$(grep "$KEY" /var/ossec/logs/alerts/alerts.json | jq -r '.[0].timestamp' | head -1)
  TETRAGON_TS=$(grep "$KEY" /var/log/tetragon/tetragon.log | jq -r '.[0].time' | head -1)

  # Compute deltas (in seconds)
  echo "  falco_ts: $FALCO_TS"
  echo "  audit_ts: $AUDIT_TS"
  echo "  wazuh_ts: $WAZUH_TS"
  echo "  tetragon_ts: $TETRAGON_TS"
done
```

## Pass Criteria

- 100% of 17 canary tokens produce events in all 4 systems within window
- 0 false positives on normal enrollment/auth/upload flows
- All events reference same `correlation_key` across all 4 systems
- All audit entries include: `auditId` (UUIDv7) + `proofHash` (64 hex chars) + `Ed25519 signature` (128 hex chars)
- No `UnimplementedError`, no `unimplemented!()`, no empty endpoint stubs
- All 4 systems' timeseries data aligns for forensic reconstruction

## Failure Modes

- **Missing falco events**: Check `eel_fingerprints.yaml` loaded, `falco` service running, kernel probe active
- **Missing audit events**: Check `audit_service` reachable, `correlation_key` in payload, `EelAttestationClient` active
- **Missing wazuh events**: Check `wazuh` agent registered, `eel_extended_rules.xml` loaded, `decoders` loaded
- **Missing tetragon events**: Check `tetragon` service running, `eel_advanced.yaml` loaded, eBPF probe active
- **Timing violations**: Increase event buffer sizes in `ossec.conf` (client_buffer queue_size)
- **False positives**: Add filter conditions to falco rule or wazuh rule

## References

- `../canary-tokens/correlation_routing.md` — full token-to-rule mapping
- `../canary-tokens/m5_drill_playbook.md` — Phase 5 details
- `docs/mobile/BACKEND_SERVICE_AUDIT_COMPLETE.md` — audit entry format
- ADR 0017 (Transparency log), ADR 0022 (Release/DR)
- NIST SP 800-53 AU-10 (Audit integrity), AU-2 (Audit events)
