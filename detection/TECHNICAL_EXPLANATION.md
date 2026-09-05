# Detection — Technical Explanation (M5 Hardening)

> Comprehensive technical explanation of EEL's 5-pillar detection stack. Why each layer exists, what it does, how they correlate, and what M5 48h drill verifies.

## 1. Why 5 Detection Layers?

A single detection system has blind spots:
- **Falco alone**: only kernel-level, no application context
- **Wazuh alone**: SIEM rules, but no kernel monitoring
- **Tetragon alone**: runtime enforcement, no historical correlation
- **Sigma rules alone**: vendor-neutral, but no enforcement
- **Canary tokens alone**: detect access, no attribution

Combining all 5 gives:
- **Coverage**: kernel + filesystem + application + network + runtime
- **Defense-in-depth**: each layer catches what others miss
- **Cross-verification**: same event in 4 systems = high confidence
- **M5 48h drill**: full chain verified end-to-end

## 2. Layer-by-Layer Breakdown

### Layer 1: Falco (Kernel Syscall Detection)
- **Mechanism**: eBPF probe (or kernel module) intercepts syscalls
- **Coverage**: 70 rules across 5 files
- **Strength**: detects kernel-level threats (root mount, kernel module unload, ptraced process)
- **Blind spot**: no application context (knows the syscall but not the user role)
- **Correlation key**: every rule outputs `correlation_key` to `audit_service` event

### Layer 2: Sigma Rules (Generic SIEM Correlation)
- **Mechanism**: vendor-neutral rules (declarative conditions on log events)
- **Coverage**: 50 rules across 9 files
- **Strength**: cross-platform (Wazuh, Splunk, Elastic, Chronicle) — same rules, multiple SIEMs
- **Blind spot**: depends on log ingestion (no kernel-level enforcement)
- **Correlation**: Sigma rule IDs map to Wazuh XML rule IDs (100110-101013)

### Layer 3: Tetragon (eBPF Runtime Enforcement)
- **Mechanism**: eBPF programs attached to kernel hooks; policies enforced before syscall completes
- **Coverage**: 15 policies across 2 files
- **Strength**: **blocking** (not just detection) — can drop syscalls, kill processes
- **Blind spot**: more complex to author than rules
- **M5 role**: enforce MDM policies, capability drops, file integrity, process isolation

### Layer 4: Wazuh (Centralized SIEM)
- **Mechanism**: agent on every device, manager aggregates, applies rules, correlates
- **Coverage**: 5 decoders + 74+ rules + 1 agent config
- **Strength**: centralized correlation, active response, SCA, vulnerability detection
- **Blind spot**: depends on agent registration and rule loading
- **M5 enhancement**: TLS/mTLS, expanded syscheck, SCA, vulnerability detection

### Layer 5: Canary Tokens (Honeypot Detection)
- **Mechanism**: fake credentials / files / endpoints / audit records embedded in build
- **Coverage**: 17 tokens across 6 namespaces (audit, biometric, crypto, network, supply chain, MDM)
- **Strength**: detects unauthorized access with high confidence
- **Blind spot**: must be deployed correctly; if not, no detection
- **M5 role**: trigger alerts via falco + wazuh + audit_service + tetragon simultaneously

## 3. Cross-Layer Correlation (the heart of M5)

Every event flow goes through 4-5 layers:

```
Event: HSM key extraction attempt
├─ Falco (kernel): rule fires with correlation_key=crypto-corr-ed25519-001
├─ Wazuh (SIEM): falco-decoder extracts event, rule id=101012 fires
├─ audit_service: eventType=crypto_access, auditId generated, proofHash + Ed25519 signed
├─ Tetragon: process_krn matches, file integrity policy violated
└─ SRE pager: EMERGENCY severity triggers PagerDuty
   → All 5 systems reference same correlation_key
   → Time delta verification: 60s/120s/60s/30s windows
   → M5 48h drill verifies this correlation
```

## 4. Why Sequential M0→M1→M5 Matters for Detection

| Stage | Detection Layer Added | Why |
|---|---|---|
| M0 | Canary tokens (baseline) | Establish honeypot detection from day 0 |
| M1 | Falco rules (kernel) | Detect kernel-level threats during enrollment |
| M2 | Sigma rules (audit) | Correlate audit events across services |
| M3 | Tetragon policies (runtime) | Enforce M3 hardware isolation at kernel |
| M4 | Wazuh extended rules (SIEM) | Centralize + correlate from all 4 systems |
| M5 | All 5 + correlation verify | 48h drill: 100% correlation, 0 false positives, M5 48h report |

Detection maturity grows with sequential progression. M5 drill verifies the **full chain** — not just individual layers.

## 5. Audit Reference (5-axis) in Detection Reports

All M5 drill reports, canary triggers, and incident responses use the same audit reference format from `docs/mobile/POLL_CLERK.md`:

```yaml
auditId: <UUIDv7>  # uuid7str()
proofHash: <SHA-256 hex, 64 chars>  # RFC 8785 canonical JSON
signature: <Ed25519 hex, 128 chars>  # EelSigningKey
correlation_key: <matches correlation_routing.md>
actorId: <who>
actorRole: <role>
stationId: <where>
whenUtc: <ISO 8601 timestamp>
M3_device_id: <M3 hardware device id>  # if hardware event
auditChainStatus: intact / broken / replayed
```

This is **the same format** used in:
- `docs/mobile/BACKEND_SERVICE_AUDIT.md`
- `docs/mobile/BACKEND_SERVICE_AUDIT_COMPLETE.md`
- `docs/mobile/POLL_CLERK.md`
- `docs/mobile/observer.md`
- `docs/mobile/party-agent.md`
- `docs/mobile/citizen-verify.md`
- All 14 mobile apps

## 6. M5 48h Drill — What It Verifies

The M5 48h drill (per `docs/mobile/M5_SMS_USB_MANIFEST.md` and `docs/mobile/M3_HARDWARE_INTEGRATION.md`) is a **complete security posture check**:

1. **Phase 1 (0-6h)**: Pre-drill verification — canary presence, correlation key registry, agent registration
2. **Phase 2 (6-12h)**: Canary activation — 17 simulated attacks, verify all 5 systems trigger
3. **Phase 3 (12-18h)**: MDM / device policy — verify FLAG_SECURE, kiosk mode, device policy intact
4. **Phase 4 (18-24h)**: Supply chain — verify APK/library hash, signature, timestamp
5. **Phase 5 (24-36h)**: Cross-system correlation — 100% of events in all 4 systems within window
6. **Phase 6 (36-48h)**: Report generation — `05_dr_report.md` template with audit reference format

**Pass criteria**:
- 100% correlation across all 4 systems for all 17 canary tokens
- 0 false positives on normal flows (enrollment, auth, upload, inspection)
- All audit events include: `auditId` + `proofHash` + `Ed25519 signature`
- Sequential rules (§11.0) M0→M1→M2→M3→M4→M5 maintained
- All references to ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023 in report

**Failure modes** (any of these = M5 drill PARTIAL PASS or FAIL):
- Correlation < 100% → drill FAIL
- False positive > 0% on normal flow → drill PARTIAL PASS
- M5 sequential rule broken → drill FAIL
- Missing audit entry (no auditId / proofHash / signature) → drill FAIL
- SRE pager not triggered for EMERGENCY → drill FAIL

## 7. Regulatory / Standards Compliance

| Standard | Reference | Detection Coverage |
|---|---|---|
| DPA 2019 Kenya §6 | Data protection / audit chain | Falco (audit_tamper), Sigma (audit_replay), Wazuh (100110-100117) |
| Elections Act §34 | Tamper-evident / form integrity | Falco (form_34a_*), Sigma (form_integrity.yaml), Wazuh (100810-100811) |
| NIST SP 800-53 AU-2 | Audit Events | All 5 layers |
| NIST SP 800-53 AU-10 | Audit Integrity | proofHash, Ed25519 sig, audit_shadow |
| NIST SP 800-53 SI-4 | System Monitoring | Falco + Tetragon + Wazuh |
| NIST SP 800-53 SC-8 | Communications Security | TLS verification (eel_net_004) |
| MITRE ATT&CK | TTP references in falco/sigma/wazuh | e.g., T1070.001, T1565.001, T1055.008 |
| ISO 27001 | A.12.4 (audit logging) | All 5 layers |

## 8. Why "No Stubs" Matters for Detection

Stubs in detection are catastrophic:
- **Empty endpoint stub** in Wazuh rules → silent failure (no alert) → undetectable breach
- **Unimplemented rule** in Falco → rules pass syntax but never fire → false negative
- **Mock audit_event** in correlation_routing.md → canary triggers but audit entry not created → chain broken
- **Empty field in audit entry** → no `auditId` / `proofHash` / `signature` → M5 drill FAILS verification

All 5 detection layers in this directory have:
- Real, working rules (`falco-rules/eel_*.yaml`, `sigma-rules/*.yaml`, `tetragon-policies/*.yaml`, `wazuh-config/rules/*.xml`)
- Real decoders (`wazuh-config/decoders/*.xml`) — not placeholders
- Real canary files referenced (`canary-tokens/distributed_canary.yaml`)
- Real correlation chains (`canary-tokens/correlation_routing.md`)
- Real playbooks (`playbooks/*.md`) — not stubs

## 9. How to Extend Detection (without breaking the chain)

1. **Add new falco rule** to `eel_extended.yaml` or new file
2. **Add sigma rule** to relevant category file or new file
3. **Add tetragon policy** to `eel_advanced.yaml` or new file
4. **Add wazuh decoder** to `decoders/` if new log format
5. **Add wazuh rule** to `rules/eel_extended_rules.xml` referencing falco rule name
6. **Add canary token** to `distributed_canary.yaml` with new `correlation_key`
7. **Update correlation_routing.md** with new mapping (token → falco/wazuh/audit/tetragon)
8. **Add playbook** to `playbooks/` for new event type
9. **Add entry in 05_dr_report.md** template for new event type
10. **Test correlation** via `playbooks/02_correlation_verify.md` before commit

Every change must:
- Reference ADR (0004, 0006, 0009, 0010, 0019, 0022, 0023 as applicable)
- Include `auditId` + `proofHash` + `Ed25519 signature` reference
- Include `correlation_key` (new or existing)
- Include `M3_device_id` for hardware rules
- Pass M5 48h drill verification

## 10. Summary

5 detection layers working in concert:
- **Falco**: kernel-level threat detection
- **Sigma**: cross-platform SIEM correlation
- **Tetragon**: eBPF runtime enforcement
- **Wazuh**: centralized SIEM + active response
- **Canary tokens**: honeypot detection

All correlated via `auditId` + `correlation_key` + `audit_event` (with proofHash + Ed25519 signature).

M5 48h drill verifies the **full chain** end-to-end with audit reference format from `docs/mobile/POLL_CLERK.md`.

Sequential rules (§11.0): M0→M1→M2→M3→M4→M5 maintained. References ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023.

No stubs. No `UnimplementedError`. No `unimplemented!()`. No empty endpoint stubs. No empty fields. No mock canary.

**Detection is the security backbone of EEL. It must be real, working, and verified.**
