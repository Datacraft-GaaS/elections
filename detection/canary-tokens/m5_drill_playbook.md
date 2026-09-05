# M5 Security Drill — Canary token activation playbook (48-hour)
# References: docs/mobile/M5_SMS_USB_MANIFEST.md, docs/mobile/M3_HARDWARE_INTEGRATION.md,
#             docs/mobile/M3_SMS_USB_MANIFEST.md, docs/adr/*
# Drill objective: verify canary tokens trigger correctly, audit chain intact,
#                  MDM policies unmodified, biometric replay blocked,
#                  supply chain integrity preserved, no false positives.

# ──────────────────────────────────────────────────────────────────────────────
# PHASE 1: PRE-DRILL VERIFICATION (0-6h)
# ──────────────────────────────────────────────────────────────────────────────

## 1.1 Canary token presence check (build-time injection verified)
# Command (on device):
#   find /data/local/eel/canary/ /data/user_de/0/com.eel.bvr/files/biometric/
# Expected: all 12+ canary files present.
# Audit reference: audit_event with eventType=build_injection_verified

## 1.2 Correlation key registry check
# Command (on audit server):
#   SELECT correlation_key FROM canary_registry WHERE activated = true;
# Expected: all 12 correlation keys present (audit-corr-vote-001, audit-corr-tally-001,
# audit-corr-form-001, bio-corr-fp-001/002, bio-corr-iris-001, crypto-corr-ed25519-001,
# crypto-corr-aes-001, crypto-corr-jwt-001, net-corr-bff-001, net-corr-iprs-001,
# net-corr-credential-001, sc-corr-library-001, sc-corr-apk-001, mdm-corr-kiosk-001,
# mdm-corr-policy-001)

# ──────────────────────────────────────────────────────────────────────────────
# PHASE 2: CANARY ACTIVATION (6-12h) — SIMULATED ATTACKS
# ──────────────────────────────────────────────────────────────────────────────

## 2.1 Audit replay simulation (simulated attacker attempts replay)
# Action: replay audit event c7d2a34e8a91b3c4d8e7f9210c3b5a6e from attacker process.
# Expected: falco ALERT + audit_service ALERT + wazuh ALERT (rule 100100 / 100102)
# Correlation chain: audit-corr-vote-001 -> falco (replay) -> audit (auditId) -> SRE alert
# Time window: 60s max from first trigger to full correlation.

## 2.2 Fake biometric template access (simulated unauthorized read)
# Action: open /data/user_de/0/com.eel.bvr/files/biometric/canary-fp-001.bin from non-app process.
# Expected: falco ALERT + audit_service ALERT + wazuh ALERT (falco rule audit_tamper / encryption / biometric_replay)
# Correlation: bio-corr-fp-001 -> falco (bio-access) -> audit (auditId = bio-tpl-access)

## 2.3 Fake HSM key access (simulated extraction attempt)
# Action: open /data/local/eel/canary/fake-ed25519-key.bin from unexpected process.
# Expected: falco EMERGENCY + audit_service EMERGENCY + SRE escalation (falco key_access / hsm_key_access)
# Correlation: crypto-corr-ed25519-001 -> falco (key_access) -> audit (crypto_access_event)

## 2.4 Network canary — fake IPRS record queried
# Action: query IPRS endpoint for IPRS-CANARY-001.
# Expected: falco ALERT (iprs_access) + audit ALERT (iprs_query) + wazuh ALERT (rule 100400 / 100400)
# Correlation: net-corr-iprs-001 -> falco (network_iprs) -> audit (auditId = iprs_verify_duplicate)

# ──────────────────────────────────────────────────────────────────────────────
# PHASE 3: MDM / DEVICE-POLICY TEST (12-18h)
# ──────────────────────────────────────────────────────────────────────────────

## 3.1 MDM policy modification attempt
# Action: modify /data/system/device_policy.xml (simulated root access).
# Expected: falco ALERT (mdm_policy_modify / device_policy_write) + audit_service ALERT
# Correlation: mdm-corr-policy-001 -> falco (mdm_policy) -> audit (auditId = mdm_policy_change)

## 3.2 FLAG_SECURE disable attempt (simulated screen capture)
# Action: send SIGTERM / SIGKILL to biometric app (simulates kiosk exit attempt).
# Expected: falco ALERT (screen_capture / kiosk_exit) + audit_service ALERT
# Correlation: mdm-corr-kiosk-001 -> falco (mdm_flag_secure) -> audit (auditId = kiosk_exit)

# ──────────────────────────────────────────────────────────────────────────────
# PHASE 4: SUPPLY CHAIN TEST (18-24h)
# ──────────────────────────────────────────────────────────────────────────────

## 4.1 Fake APK modification (simulated repackaging)
# Action: modify binary at /data/app/com.eel.bvr/lib/canary-libbvr.so
# Expected: falco EMERGENCY (supply_chain_mod) + audit EMERGENCY (build_integrity_fail)
# Correlation: sc-corr-library-001 -> falco (supply_chain) -> audit (auditId = supply_chain_integrity_fail)

## 4.2 Signature verification (simulated backdated binary)
# Action: check binary timestamp (future date detection).
# Expected: falco ALERT (supply_chain_future_timestamp) + audit ALERT
# Correlation: sc-corr-library-001 -> falco (future_timestamp) -> audit (auditId = future_timestamp_detected)

# ──────────────────────────────────────────────────────────────────────────────
# PHASE 5: FULL CORRELATION TEST (24-36h)
# ──────────────────────────────────────────────────────────────────────────────

## 5.1 Cross-system correlation (falco + wazuh + audit_service + tetragon)
# For each triggered event (audit replay, biometric replay, crypto access, MDM violation,
# supply chain violation, network canary), verify:
#
#  - falco ALERT emitted within 60s of trigger.
#  - audit_service event created within 120s of falco alert (correlation_key matched).
#  - wazuh ALERT emitted within 60s of audit event.
#  - tetragon event emitted within 30s of trigger.
#  - All 4 sensors reference the same correlation_key and auditId.
#  - No false positives from normal biometric enrollment/auth/upload operations.
#
# Success criteria: 100% correlation for all 12 canary types, 0 false positives on normal flow.

# ──────────────────────────────────────────────────────────────────────────────
# PHASE 6: M5 48h DRILL REPORT (36-48h)
# ──────────────────────────────────────────────────────────────────────────────

## 6.1 Drill report template (must include audit reference)
# Required fields per audit reference (docs/mobile/POLL_CLERK.md style):
#  - where: detection node / station / M5 drill phase
#  - when: UTC timestamp (ISO 8601)
#  - who: operator / auditor / SRE (actorId + actorRole)
#  - why: M5 48-hour security drill (eventType = m5_drill)
#  - how: canary token trigger + correlation chain
#  - proofHash: audit entry hash
#  - signature: Ed25519 signature
#  - auditId: canary-correlation audit event
#  - correlationKey: links to canary token
#  - M3_device_id: device identifier
#  - devicePolicyStatus: MDM status
#  - auditChainStatus: intact / broken / replayed
#  - telemetryStatus: falco / wazuh / tetragon / audit_service all present
#  - securityStatus: canary_triggered / canary_intact / breach_detected / false_positive / nothing_detected
#
# Report must reference ADR 0004 (Platform), ADR 0006 (Build supply chain),
# ADR 0010 (Edge/DDoS), ADR 0019 (Topology), ADR 0022 (Release/DR), ADR 0023 (RFP alignment).
# Report must include sequential M5 reference (§11.0 M5 after M4 gates: MDM rollout, rooted AVD, audit shadow, FLAG_SECURE, a11y audit, 48h drill).
