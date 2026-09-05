# Detection Playbooks — M5 48-Hour Hardening Drill

> Operational playbooks for M5 security audit: canary activation, correlation verification, drift detection, incident response, DR report generation. All playbooks reference `correlation_routing.md`, `m5_drill_playbook.md` (canary-tokens/), and audit reference format from `docs/mobile/POLL_CLERK.md`.

## Playbook List

| Playbook | Phase | Purpose | References |
|---|---|---|---|
| `01_canary_activation.md` | Phase 2 (6-12h) | Simulate attacks against 17 canary tokens; verify alert routing | canary-tokens/m5_drill_playbook.md |
| `02_correlation_verify.md` | Phase 5 (24-36h) | Cross-system correlation: falco + wazuh + audit + tetragon | correlation_routing.md |
| `03_drift_detection.md` | Phase 3 (12-18h) | Detect MDM / device-policy / flag_secure changes | mdm_policy.yaml, falco-rules/eel_fingerprints.yaml |
| `04_incident_response.md` | Emergency (any time) | Response to critical alerts (audit replay, HSM access, root mount) | falco-rules, wazuh-config/ossec.conf |
| `05_dr_report.md` | Phase 6 (36-48h) | Generate M5 48h drill report with audit reference format | docs/mobile/IMPLEMENTATION_COMPLETE.md |

## Usage

```bash
# Phase 2: Activate canary (simulated)
cat playbooks/01_canary_activation.md | bash -

# Phase 5: Verify correlation
cat playbooks/02_correlation_verify.md | bash -

# All phases produce output referencing auditId + correlation_key + M3_device_id
```

All playbooks require: `audit_service` access, `falco` running, `wazuh` agent registered, `tetragon` agent active, and `correlation_routing.md` loaded.
