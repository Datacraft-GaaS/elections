# Wazuh Agent Config — SIEM / Detection Integration

> Central SIEM at `wazuh.eel.local`. Agents on biometric/kit devices, servers, BFF nodes. M5 hardening: TLS/mTLS, expanded syscheck, vulnerability scanning, SCA, canary correlation, audit event emission.

## What

**Wazuh** collects, correlates, and alerts on security events from the EEL election infrastructure. Agent runs on every biometric device, server, and BFF node. Manager (`wazuh.eel.local`) aggregates, applies rules, correlates, and triggers active response.

## Files

| File | Purpose | Rules/Decoders |
|---|---|---|
| `ossec.conf` | Agent configuration: TLS, syscheck, rootcheck, SCA, vulnerability detector, active-response, audit emission, M5 drill labels | 1 agent config |
| `decoders/eel_audit_decoder.xml` | EEL audit JSON events (auditId, eventType, actorId) | 1 decoder |
| `decoders/eel_biometric_decoder.xml` | Biometric capture events (sessionId, biometricType, qualityScore) | 1 decoder |
| `decoders/eel_outbox_decoder.xml` | Outbox events (submissionId, eventType, status) | 1 decoder |
| `decoders/falco_decoder.xml` | Falco events (priority, rule, output, proc.name) | 1 decoder |
| `decoders/tetragon_decoder.xml` | Tetragon events (process_krn, process_path) | 1 decoder |
| `decoders/operator_decoder.xml` | Operator/auth events (actorId, eventType, stationId) | 1 decoder |
| `rules/eel_rules.xml` | Base EEL rules (14 rules — audit integrity, biometric quality, outbox) | 14 |
| `rules/eel_extended_rules.xml` | Extended M5 rules (60+ rules — audit replay, election integrity, biometric/hardware M3, MDM, supply chain, network/DDoS, session/operator, form integrity, outbox, canary triggers) | 60+ |

**Total: 8 files, 5 decoders, 74+ rules, 1 agent config.**

## Design Principles

- **No stubs** — all rules have `id`, `level`, `description`, `decoded_as`, `mitre`, `tags`, `fields`
- **Every rule references real sources**: `falco-decoder`, `eel-audit-decoder`, `eel-biometric-decoder`, `operator-decoder`, `tetragon-decoder`
- **Audit format preserved**: all rules extract `auditId`, include `correlation_key` reference, `M3_device_id` (hardware), `actorId`, `actorRole`
- **M5 drill correlation**: `correlationKey` field maps to `correlation_routing.md` for cross-system verification
- **Active response**: `firewall-drop` (IP ban for 600s) + `eel-audit-route` (audit event routing) triggered by critical rules
- **Agent groups**: `eel_biometric`, `eel_m5_drill`, `eel_canary`, `eel_audit` — allows targeted rule application

## Agent Security

- **TLS/mTLS** to manager (`wazuh.eel.local`) using CA + agent certificates
- **Enrollment** with `agent_address=auto` — auto-discovers manager
- **Client buffer**: 10,000 events / 1000 per second — handles M5 drill load
- **White list**: internal subnets only (10/172/192 + localhost)

## Expanded Syscheck (M5)

Beyond base audit paths:
- **Canary tokens**: `/data/local/eel/canary/` + all 11 token files (fake keys, templates, URLs, credentials)
- **HSM key**: `/etc/eel/keys/signing_key.bin` + `/dev/hsm`
- **Biometric templates (encrypted)**: 3 canary files + encrypted directories
- **Form 34A/B/C**: `/data/local/eel/forms/`
- **MDM policy**: `/data/system/device_policy*.xml`
- **BFF config / API key**: `/etc/eel/bff/`
- **Library integrity**: `libflutter.so`, `libbvr.so`, `libbvi.so`, `librts.so` + canary library
- **Ignore**: `/data/local/tmp`, `/var/log` (noise reduction)

## SCA (Security Configuration Assessment)

4 policies:
- `eel_election_hardening.yml` — election-specific hardening
- `eel_biometric_hardening.yml` — biometric device hardening
- `cis_linux.yml` — CIS Linux Benchmark
- `cis_android.yml` — CIS Android Benchmark

## Vulnerability Detection

- `canonical` (Ubuntu/Debian): `trusty`, `xenial`, `bionic`, `focal`
- `nvd`: US NVD vulnerability database
- Scan interval: 5h, on start

## References

- ADR 0004 (Platform): agent group design + hardening
- ADR 0009 (Resilience posture): vulnerability detection + SCA
- ADR 0019 (Edge topology): BFF endpoint correlation, GPS, network allowlist
- ADR 0017 (Transparency log): audit chain / audit_event emission
- DPA 2019 Kenya §6: audit integrity / data protection
- Elections Act §34: tamper-evident logging / form 34A integrity
- M3-HARDWARE-INTEGRATION: M3 device paths / hardware rules
- NIST SP 800-53 AU-2, SI-4, SC-8, AU-10
