# Sigma Rules — Generic SIEM Correlation (Cross-Platform)

> Sigma is a generic, open signature format for SIEM queries (Splunk, Elastic, Chronicle, Microsoft Sentinel, Wazuh). Rules are converted to target SIEM query language.

## What

Sigma rules define **correlation conditions** for audit, biometric, election integrity, supply chain, MDM, network, hardware, form integrity events. Each rule is independent of SIEM vendor — only `logsource` (product/service) and `detection` (selection/filter/condition) are declared.

## Files

| File | Rules | Domain |
|---|---|---|
| `biometric_audit_rules.yaml` | 13 | Audit chain (proof hash, Ed25519 sig), biometric quality, outbox, partition, IPRS duplicate, MRZ |
| `election_integrity.yaml` | 5 | Double voting, bulk enrollment, off-hours, IPRS duplicate, GPS mismatch |
| `supply_chain.yaml` | 4 | APK sig fail, library mismatch, future timestamp, APK modification |
| `mdm_policy.yaml` | 3 | Device policy modify, FLAG_SECURE disable, root partition |
| `network_exfil.yaml` | 5 | Unexpected outbound, BFF rate limit, IPRS off-hours, TLS untrusted CA, non-allowlisted DNS |
| `audit_replay.yaml` | 5 | Replay event, replay script, proof hash mismatch, signature fail, audit shadow modify |
| `operator_access.yaml` | 4 | Role escalation, brute force auth, cross-station audit access, session token copy |
| `hardware_m3.yaml` | 6 | FP/iris quality, FAP60/AntIris unauthorized, GSM unauthorized, RFID unauthorized |
| `form_integrity.yaml` | 5 | Form 34A post-signature modify, unauthorized access, upload fail, invalid Ed25519 key, off-hours submission |

**Total: 50 rules across 9 files.**

## Design Principles

- **Generic**: no Splunk/Elastic-specific syntax — only `detection.selection`, `filter`, `condition`
- **Every rule includes**: `title`, `id`, `status` (`production` / `critical` / `alert` / `warning`), `references` (ADR + regulatory + MITRE), `tags`, `fields`, `level`
- **Reference mapping**: `id: eel-...-###` maps to Wazuh rule IDs 100110-101013 (see `correlation_routing.md`)
- **No stubs**: all rules reference real event sources (`audit_service`, `syslog`, `auditd`, `syscall`, `auth_service`, `form_service`, `biometric_capture`, `build_integrity`, `iprs_client`)
- **Audit format preserved**: all fields include `auditId`, `correlation_key`, `M3_device_id` (hardware rules), `actorId`, `actorRole`, `stationId`, `whenUtc`

## Correlation Chain (Sigma → Wazuh → Falco)

```
Sigma rule (id: eel-ei-001) → Wazuh (id: 100210)
→ matches audit_service event (eventType: enrollment) → falco ALERT (audit_tamper)
→ audit_service writes audit event (auditId + proofHash + Ed25519 signature)
→ audit_shadow updated → M5 48h drill verified
```

## Regulatory References

- DPA 2019 Kenya §6 (data protection / audit chain)
- Elections Act §34 (tamper-evident / form integrity)
- NIST SP 800-53 AU-10 (audit integrity), AU-2 (audit events), SI-4 (monitoring)
- ADR 0004 (Platform), ADR 0017 (Transparency log), ADR 0022 (Release/DR), ADR 0023 (RFP alignment)

## Usage

```bash
# Convert Sigma to target SIEM (example for Splunk / Elastic / Wazuh)
# Rules are already mapped to Wazuh XML in ../wazuh-config/rules/
# For Splunk: sigma-py or sigmac conversion
```

See `docs/mobile/BACKEND_SERVICE_AUDIT_COMPLETE.md` for audit reference format used by all sigma rules.
