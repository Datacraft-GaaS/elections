# Falco Rules — Kernel / Syscall Threat Detection

> Kernel-level threat detection for EEL biometric infrastructure. Ships to `wazuh.eel.local` and `audit.eel.local`.

## What

**Falco** monitors Linux kernel syscalls via an eBPF probe or kernel module. Rules fire on syscall patterns that match threat indicators. Output routes to Wazuh SIEM, audit_service, and SRE alerting.

## Files

| File | Rules | Coverage |
|---|---|---|
| `eel_fingerprints.yaml` | 18 | Core biometric / kernel: audit tamper, HSM/keystore access, hardware (FAP60/AntIris/GSM), kernel module unload, root/jailbreak, network exfil, process injection, biometric session |
| `eel_siem.yaml` | 6 | Election integrity: double-voting, off-hours enrollment, kiosk mode violation, bulk enrollment, GPS mismatch, form 34A/B/C tampering |
| `eel_extended.yaml` | 18 | M5 extension: audit replay/rebuild, outbox AES-256 access, MDM/FLAG_SECURE, supply chain APK integrity, session hijacking, BFF proxy/API tampering, operator role escalation + auth brute-force, biometric replay |
| `eel_network_ddos.yaml` | 12 | Network: BFF rate-limit/DDoS, IPRS direct access, audit event smuggling (oversized/invalid proof_hash/signature), port scan/lateral movement, DNS/C2 exfil, TLS untrusted CA/cert expiry |
| `eel_hardware_integrity.yaml` | 16 | M3 hardware: FAP60 disconnect/reconnect/quality, AntIris disconnect/reconnect/quality, GSM modem/SMS/AT injection, RFID location mismatch, USB hotplug, TEE/HSM access, evidence signing |

**Total: 70 rules across 5 files.**

## Design Principles

- **No stubs** — every rule matches a real syscall pattern (`openat`, `execve`, `delete_module`, `open_write`, `dns_query`, `ptrace`, `open_read`, `rename`, `unlink`, `kill`, `signal`, `create_symlink`)
- **Every rule outputs**: `node=%hostname`, `user=%user.name`, `user.uid`, `terminal`, `command`, `correlation_key` (links to `correlation_routing.md`), `M3_device_id` (hardware rules)
- **Severity**: `EMERGENCY(0)` → `ALERT(1)` → `CRITICAL(2)` → `ERROR(3)` → `WARNING(4)` → `NOTICE(5)` → `INFO(6)` → `DEBUG(7)`
- **Audit correlation**: every output includes a `correlation_key` matching a canary token or sigma rule for cross-system correlation at M5 48h drill

## Severity Taxonomy

| Level | Meaning in EEL context |
|---|---|
| EMERGENCY | Kernel monitoring disabled, root partition RW remount, HSM key extraction |
| CRITICAL | Audit chain broken, biometric session hijacked, supply chain modified, hardware swap detected |
| ALERT | Unexpected outbound/C2, role escalation, off-hours enrollment, kiosk mode violated |
| WARNING | Quality below threshold, auth retries exceeded, non-allowlisted DNS |

## M3 Hardware Integration

All hardware rules reference real M3 device paths:
- **FAP60**: `/dev/usb`, `fap60`, `FUFingerSDK` (fingerprint)
- **AntIris**: `antiris`, `AntUSBCamera`, `irex` (iris)
- **GSM modem**: `/dev/ttyUSB`, `/dev/ttyACM`, `/dev/usb` (SMS)
- **RFID**: `hidraw`, `rfid`, `ttyUSB` (RTS reader)
- **TEE/HSM**: `/dev/tee`, `/dev/optee`, `/dev/hw_random`, `/dev/hsm`

## References

- ADR 0004 (Platform): sensor integrity + tamper-evident chain
- ADR 0006 (Build & supply chain): APK/library integrity, build injection
- ADR 0010 (Edge & DDoS): network rate-limit, C2 detection
- ADR 0019 (Regional edge topology): GPS mismatch, station polygon, network allowlist
- DPA 2019 Kenya §6: audit chain, HSM keys, biometric data
- Elections Act §34: tamper-evident forms, audit integrity
- NIST SP 800-53 AU-2 (Audit Events), SI-4 (System Monitoring), SC-8 (Communications Security)
- MITRE ATT&CK: TTP-T0812, TTP-T1014, TTP-T1548, TTP-T1526, TTP-T1071, TTP-T1055, TTP-T1059, TTP-T1046

## Correlation

Rule → `correlation_key` → `audit_routing.md` → Wazuh rule + audit_service event + tetragon policy + SRE alert.

See `docs/mobile/M3_HARDWARE_INTEGRATION.md` and `docs/mobile/M3_SMS_USB_MANIFEST.md` for M3 hardware manifest.
