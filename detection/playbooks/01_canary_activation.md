# Playbook 01: Canary Token Activation (M5 Drill Phase 2)

> Simulated attack against 17 canary tokens to verify alert routing. Each canary must trigger:
> - `falco` ALERT (within 60s)
> - `audit_service` event (within 120s) with `auditId` + `proofHash` + `Ed25519` signature
> - `wazuh` ALERT (within 60s) with `decoded_as` reference
> - `tetragon` event (within 30s) with `process_krn` reference
> - `sre_pager` (for EMERGENCY tokens only)

## Prerequisites

- `audit_service` running at `audit.eel.local`
- `falco` with rules loaded
- `wazuh` agent registered, `eel_audit` group enabled
- `tetragon` agent active
- `correlation_routing.md` loaded into central detection
- All 17 canary files present in `/data/local/eel/canary/`, `/data/user_de/0/com.eel.bvr/files/biometric/`, etc.

## Phase 2.1 — Audit Replay (canary-audit-vote-001)

**Trigger:** `auditId=c7d2a34e8a91b3c4d8e7f9210c3b5a6e` is replayed.

```bash
# 1. Submit fake audit query (simulates replay attempt)
curl -X POST "https://bvr.eel.local/api/audit/query" \
  -H "Authorization: Bearer canary-test-token" \
  -H "Content-Type: application/json" \
  -d '{"auditId":"c7d2a34e8a91b3c4d8e7f9210c3b5a6e","stationId":"station-canary-001"}'

# Expected response: 200 OK with "canary_triggered" status
```

**Verify (within 60s):**
1. `falco` ALERT: `rule="Audit replay script executed"` with `correlation_key=audit-corr-vote-001`
2. `audit_service` event: `eventType=audit_replay`, `auditId=c7d2a34e...`, `proofHashStatus=mismatch`
3. `wazuh` ALERT: `rule id=100112`, level=14, MITRE T1070.001
4. `tetragon` event: `process_krn` referencing the query process

**Pass criteria:** All 4 systems emit ALERT within 60s, all reference `correlation_key=audit-corr-vote-001` and `auditId=c7d2a34e...`.

## Phase 2.2 — Fake Biometric Template Access (canary-bio-fp-001)

**Trigger:** Open `/data/user_de/0/com.eel.bvr/files/biometric/canary-fp-001.bin` from non-app process.

```bash
# 1. Simulate unauthorized read
echo "simulated" > /tmp/test_canary
cp /data/user_de/0/com.eel.bvr/files/biometric/canary-fp-001.bin /tmp/test_canary_read
cat /tmp/test_canary_read  # triggers read
```

**Verify (within 60s):**
1. `falco` ALERT: `rule="Outbox encrypted bundle accessed by unexpected process"` with `correlation_key=bio-corr-fp-001`
2. `audit_service` event: `eventType=outbox_access_unexpected`, `biometricType=fingerprint`
3. `wazuh` ALERT: `rule id=100912` (or `100310` for quality), level=14
4. `tetragon` event: `process_krn` referencing the read process

**Pass criteria:** All 4 systems emit ALERT, all reference `correlation_key=bio-corr-fp-001`.

## Phase 2.3 — Fake HSM Key Access (canary-crypto-ed25519-001)

**Trigger:** Open `/data/local/eel/canary/fake-ed25519-key.bin` from non-attestation process.

```bash
# 1. Simulate extraction
cat /data/local/eel/canary/fake-ed25519-key.bin
# OR
cp /data/local/eel/canary/fake-ed25519-key.bin /tmp/
```

**Verify (within 30s):** This is **EMERGENCY** severity.
1. `falco` EMERGENCY: `rule="HSM key reference accessed but key not found"` or `"Ed25519 signing key accessed from device"`, `correlation_key=crypto-corr-ed25519-001`
2. `audit_service` EMERGENCY: `eventType=crypto_access`, `keyType=ed25519`
3. `wazuh` ALERT: `rule id=101012`, level=14
4. `tetragon` EMERGENCY: `process_krn` referencing the process
5. `sre_pager` triggered within 30s

**Pass criteria:** All 5 systems emit EMERGENCY/ALERT, all reference `correlation_key=crypto-corr-ed25519-001`. **SRE pager MUST be triggered.**

## Phase 2.4 — Network Canary (canary-net-bff-001)

**Trigger:** Request `https://bff-canary.eel.local/canary-ping`.

```bash
curl -X GET "https://bff-canary.eel.local/canary-ping"
```

**Verify:**
1. `falco` ALERT: `rule="BFF request to non-standard path"` with `correlation_key=net-corr-bff-001`
2. `audit_service` event: `eventType=bff_request`, `endpoint=/canary-ping`
3. `wazuh` ALERT: `rule id=100716` (BFF payload altered) or `100710` (rate limit)
4. `tetragon` event: network policy violation

## Phase 2.5 — Supply Chain (canary-sc-library-001)

**Trigger:** Modify `/data/app/com.eel.bvr/lib/canary-libbvr.so` (simulated).

```bash
# Simulate modification (in test environment only)
echo "test" >> /data/app/com.eel.bvr/lib/canary-libbvr.so
# OR load it from non-expected process
ldd /data/app/com.eel.bvr/lib/canary-libbvr.so
```

**Verify:**
1. `falco` EMERGENCY: `rule="APK / library replaced with different hash"` with `correlation_key=sc-corr-library-001`
2. `audit_service` EMERGENCY: `eventType=library_integrity_check`, `HashStatus=mismatch`
3. `wazuh` ALERT: `rule id=100510`, level=14
4. `tetragon` EMERGENCY: file integrity policy violation
5. `sre_pager` triggered

## Phase 2.6 — MDM (canary-mdm-kiosk-001)

**Trigger:** Send `SIGKILL` to biometric app (simulates kiosk exit).

```bash
# Get biometric app PID
BVR_PID=$(pidof com.eel.bvr)
# Send signal
kill -9 $BVR_PID
```

**Verify (within 60s):**
1. `falco` CRITICAL: `rule="FLAG_SECURE disabled or cleared"` with `correlation_key=mdm-corr-kiosk-001`
2. `audit_service` event: `eventType=kiosk_exit`, `kioskStatus=violated`
3. `wazuh` ALERT: `rule id=100410`, level=14
4. `tetragon` event: process policy violation

## Pass Criteria Summary

| Token | Severity | Correlation Key | 4-Systems | SRE Pager |
|---|---|---|---|---|
| canary-audit-vote-001 | CRITICAL | audit-corr-vote-001 | ✅ | optional |
| canary-audit-tally-001 | CRITICAL | audit-corr-tally-001 | ✅ | optional |
| canary-audit-form-001 | CRITICAL | audit-corr-form-001 | ✅ | optional |
| canary-bio-fp-001 | CRITICAL | bio-corr-fp-001 | ✅ | optional |
| canary-bio-fp-002 | CRITICAL | bio-corr-fp-002 | ✅ | optional |
| canary-bio-iris-001 | CRITICAL | bio-corr-iris-001 | ✅ | optional |
| canary-bio-mrz-001 | WARNING | bio-corr-mrz-001 | ✅ | optional |
| canary-crypto-ed25519-001 | **EMERGENCY** | crypto-corr-ed25519-001 | ✅ | **MUST** |
| canary-crypto-aes-001 | CRITICAL | crypto-corr-aes-001 | ✅ | optional |
| canary-crypto-jwt-001 | CRITICAL | crypto-corr-jwt-001 | ✅ | optional |
| canary-net-bff-001 | CRITICAL | net-corr-bff-001 | ✅ | optional |
| canary-net-iprs-001 | CRITICAL | net-corr-iprs-001 | ✅ | optional |
| canary-net-credential-001 | CRITICAL | net-corr-credential-001 | ✅ | optional |
| canary-sc-library-001 | **EMERGENCY** | sc-corr-library-001 | ✅ | **MUST** |
| canary-sc-apk-001 | **EMERGENCY** | sc-corr-apk-001 | ✅ | **MUST** |
| canary-mdm-kiosk-001 | CRITICAL | mdm-corr-kiosk-001 | ✅ | optional |
| canary-mdm-policy-001 | CRITICAL | mdm-corr-policy-001 | ✅ | optional |

## Failure Modes

- **No falco ALERT** → Check `falco` rules loaded, audit `/var/log/falco/events.log`
- **No wazuh ALERT** → Check `decoders/eel_audit_decoder.xml` and rules loaded, agent registered to `eel_audit` group
- **No tetragon event** → Check `tetragon` agent active, `eel_advanced.yaml` loaded
- **No audit_service event** → Check `audit_service` reachable, `correlation_key` in event payload
- **No SRE pager** (EMERGENCY only) → Check `sre_pager` config, PagerDuty service

## References

- `../canary-tokens/m5_drill_playbook.md` — phase 2 detailed playbook
- `../canary-tokens/correlation_routing.md` — token-to-rule mapping
- `../docs/mobile/POLL_CLERK.md` — audit reference format
- ADR 0004, 0006, 0010, 0019, 0022, 0023
