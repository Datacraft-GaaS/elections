# Playbook 05: M5 48h Drill Report — Audit Reference (Phase 6)

> Final report template for M5 48-hour security audit. Must include audit reference format (`where`/`when`/`who`/`why`/`how` + `proofHash` + `signature`) and reference ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023.

## Report Template

```
EEL M5 48-HOUR SECURITY DRILL REPORT
======================================
Drill ID: M5-{timestamp}
Reference: docs/mobile/M5_SMS_USB_MANIFEST.md
Audit Chain Status: intact / replayed / broken / false_positive / nothing_detected
Sequential Rules (§11.0): M0->M1->M2->M3->M4->M5 maintained: yes / partial / no

SECTION 1: DRILL OVERVIEW
--------------------------
Where: detection node / station / M5 drill phase (e.g., Phase 2 canary activation, Phase 5 correlation)
When: {whenUtc} (ISO 8601 UTC)
Who: {actorId} / {actorRole} (operator / auditor / SRE / M5 auditor)
Why: M5 48-hour security audit (eventType = m5_drill)
How: canary token activation + correlation verification + MDM/device-policy check + audit replay test
Proof Hash: {proofHash} (SHA-256, 64 hex chars — RFC 8785 canonical JSON)
Signature: {signature} (Ed25519, 128 hex chars — EelAttestationClient / EelSigningKey)
Audit ID: {auditId} (UUIDv7 — uuid7str())
Correlation Key: {correlation_key} (matches correlation_routing.md entry)
M3 Device ID: {M3_device_id} (device identifier from M3 hardware integration)
Device Policy Status: MDM rollout completed / partial / none
Audit Chain Status: intact / replayed / broken / false_positive / nothing_detected
Telemetry Status: falco / wazuh / tetragon / audit_service — all present / partial / missing
Security Status: canary_triggered / canary_intact / breach_detected / false_positive / nothing_detected

SECTION 2: CANARY RESULTS (all 17 tokens)
----------------------------------------
For each token in distributed_canary.yaml + honeytokens.yaml:
- Token ID / correlation_key
- Triggered: yes / no
- Severity: EMERGENCY / CRITICAL / ALERT / WARNING / INFO
- Falco ALERT: yes / no + time delta
- Audit service event: yes / no + auditId + proofHash verified
- Wazuh ALERT: yes / no + rule id
- Tetragon event: yes / no
- SRE Pager (if EMERGENCY): yes / no
- False Positive: yes / no
- Correlation verified: yes / no

SECTION 3: MDM / DEVICE POLICY RESULTS
--------------------------------------
- FLAG_SECURE / kiosk mode: intact / violated / disabled
- Device policy file: intact / modified / missing
- Root filesystem RW: yes / no (if yes = EMERGENCY + M5 drill FAIL)
- MDM agent: running / stopped / removed
- Kiosk mode: active / disabled / violated
- Screen capture protection: enabled / disabled

SECTION 4: SUPPLY CHAIN / BUILD INTEGRITY RESULTS
------------------------------------------------
- APK / library hash verification: all passed / some failed / none verified
- Signature verification: all passed / some failed
- Build timestamp verification: all future / some future / all valid
- APK modification (post-install): detected / none / false positive
- Canary library (sc-corr-library-001): intact / triggered / not present
- Canary APK (sc-corr-apk-001): intact / triggered / not present

SECTION 5: HARDWARE M3 RESULTS
-------------------------------
- FAP60 sensor: connected / disconnected / unexpected reconnect / unexpected access
- AntIris sensor: connected / disconnected / unexpected reconnect / unexpected access
- GSM modem: intact / unauthorized access / AT injection / SMS to non-whitelist / disconnect
- RFID reader: intact / unauthorized access / location mismatch
- TEE access: intact / unauthorized access / HSM key missing / HSM unauthorized
- USB hotplug: detected / none / false positive
- Evidence file (evidence upload): intact / modified / missing

SECTION 6: NETWORK / DDoS / EXFIL RESULTS
------------------------------------------
- BFF rate limit: exceeded / within limit / false positive
- Unexpected outbound: detected / none / false positive
- Non-allowlisted DNS: detected / none / false positive
- TLS untrusted CA: detected / none / false positive
- Port scan: detected / none / false positive
- IPRS off-hours: detected / none / false positive

SECTION 7: FORM INTEGRITY RESULTS
----------------------------------
- Form 34A/B/C: intact / modified / missing / unauthorized access / upload failure
- Form 34A signature verification: passed / failed
- Form 34A upload: success / failure / critical failure
- Form 34A off-hours submission: detected / none / false positive

SECTION 8: OPERATOR / SESSION RESULTS
--------------------------------------
- Operator role escalation: detected / none / false positive
- Auth brute-force: detected / none / false positive
- Cross-station audit access: detected / none / false positive
- Session replay: detected / none / false positive
- Session token copy: detected / none / false positive
- Kiosk mode violation: detected / none / false positive

SECTION 9: AUDIT CHAIN INTEGRITY RESULTS
-----------------------------------------
- Audit chain intact: yes / no (if no = M5 drill FAIL)
- Proof hash mismatch: detected / none / false positive
- Audit replay: detected / none / false positive
- Audit shadow modify: detected / none / false positive
- Ed25519 signature verification: all passed / some failed
- Audit service response time: within SLA / exceeded
- Audit event count (48h): total / canary-triggered / false-positive
- Audit event completeness: all events include auditId + proofHash + signature / partial / missing

SECTION 10: CORRELATION RESULTS
-------------------------------
- Cross-system correlation verified: 100% / partial / 0% (if <100% = M5 drill FAIL)
- Correlation keys matched: list of matched / unmatched
- False positive rate: 0% / partial / high (if >0% = M5 drill PARTIAL PASS)
- SRE pager triggered (for EMERGENCY): yes / no + response time
- Sequential rules (§11.0) maintained: yes / partial / no

SECTION 11: REFERENCES
----------------------
- ADR 0004 (Platform): device isolation, MDM, audit chain
- ADR 0006 (Build & supply chain): build integrity, APK/library verification
- ADR 0009 (Resilience posture): vulnerability detection, SCA, M5 hardening
- ADR 0010 (Edge & DDoS): network rate limit, C2 detection, TLS verification
- ADR 0017 (Transparency log): audit replay, proof hash, Ed25519 signature
- ADR 0019 (Regional edge topology): GPS/station polygon, BFF endpoint, IPRS
- ADR 0022 (Release & DR): M5 sequential rules, M5 48h drill
- ADR 0023 (RFP alignment): form integrity, MDM rollout, audit chain
- M3 SMS USB MANIFEST: M3 hardware integration
- M3 HARDWARE INTEGRATION: FAP60, AntIris, GSM, RFID, TEE, HSM
- DPA 2019 Kenya §6
- Elections Act §34
- NIST SP 800-53 AU-2, AU-10, SI-4, SC-8
- MITRE ATT&CK: TTP references (from falco rules / sigma rules)

SECTION 12: SIGN-OFF
--------------------
Audit Reference (5-axis):
- Where: M5 48-hour security audit site / station / detection node
- When: {whenUtc}
- Who: M5 Auditor / SRE / Operator (actorId + actorRole)
- Why: M5 security audit — verification of M0→M1→M2→M3→M4→M5 sequential rules
- How: canary activation + correlation verification + MDM/device check + audit replay + supply chain verification + form integrity + network/DDoS + operator/session + hardware M3
- Proof Hash: {proofHash}
- Signature: {signature}
- Audit Chain Status: intact / broken / replayed
- M5 Drill Status: PASS / PARTIAL PASS / FAIL
- Sequential Rules (§11.0): maintained / partial / broken
- Correlation Percentage: 100% / {percent}%
- False Positive Rate: 0% / {percent}%
- References: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023
- Audit Service Reference: docs/mobile/BACKEND_SERVICE_AUDIT_COMPLETE.md
- Implementation Complete: docs/mobile/IMPLEMENTATION_COMPLETE.md
- Implementation Summary: docs/mobile/IMPLEMENTATION_SUMMARY.md
