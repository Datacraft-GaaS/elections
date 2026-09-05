# Tetragon Policies — eBPF Runtime Security (M5 Hardening)

> Tetragon uses eBPF probes to enforce process/file/network policies at kernel level — before syscall completes. Used for M3 hardware isolation, biometric process containment, MDM enforcement.

## What

**Tetragon** monitors process creation, file access, network connections, syscall execution, mount operations, capability changes, cgroup assignments. Policies define what is allowed / denied and emit telemetry to `audit.eel.local`.

## Files

| File | Policies | Domain |
|---|---|---|
| `eel_tetragon_policy.yaml` | 7 | Process/file/network/capability/syscall/file_integrity/audit emission (base policies) |
| `eel_advanced.yaml` | 8 | Process-ancestor validation, mount restriction, PIDFD tracking, SECCOMP syscall filtering, CGroups isolation, advanced network policy (443/8443/8444 + allowed IPs), full trace/audit emission, gRPC service mesh restriction |

**Total: 15 policies.**

## Design Principles

- **Namespace-aware**: all policies specify `namespace: [host, container, kubernetes]`
- **Binary-restricted**: only `libflutter.so` (BVR/BVI) can access protected paths / devices
- **No empty policies**: every policy has `namespace_selector`, `binary_selector`, `match_policies`
- **Audit emission**: all events emit to `https://audit.eel.local/v1/telemetry` (auth_token_ref: `eel-audit-token`)
- **M5 drill correlation**: all policies reference `auditId` + `correlation_key` + `M3_device_id`

## Key Policies

| Policy | Purpose | M5 Relevance |
|---|---|---|
| `process_selector` | Only biometric apps can execute from expected paths | Prevents malware injection via binary replacement |
| `file_selector` | Restrict access to `/data/data/com.eel.bvr/`, `/data/user_de/0/com.eel.bvr/` | Prevents unauthorized biometric data access |
| `network_selector` | Only allow BFF/IPRS/audit endpoints on TCP 443/8443/8444 | Prevents exfil / C2 / unauthorized connections |
| `capability_selector` | Drop `CAP_SYS_ADMIN`, `CAP_SYS_PTRACE`, `CAP_SYS_MODULE`, `CAP_MKNOD` | Prevents privilege escalation / rootkit |
| `syscall_filter` | Block `execve`, `delete_module`, `ptrace`, `mount`, `reboot` | Prevents kernel tampering / rootkit |
| `mount_selector` (advanced) | Prevent remount of `/data`, `/data/user_de` with `rw`/`noexec` removed | Prevents filesystem compromise |
| `process_ancestor_selector` | Biometric app must have expected parent (not shell) | Prevents injection / RCE |
| `pidfd_selector` | Detect PID reuse after process close | Prevents session replay / PID reuse attack |
| `seccomp_selector` | Restrict dangerous syscalls for biometric apps | Defense in depth |
| `cgroup_selector` | Isolate biometric apps to `eel_bvr`/`eel_bvi` groups | Isolation / containment |
| `network_policy_selector` (advanced) | Enforce allowed destinations / ports | Network segmentation |
| `trace_selector` | Full telemetry emission for M5 drill | Audit correlation / verification |
| `grpc_selector` (advanced) | Restrict BVR/BVI/RTS gRPC to allowed methods | Service mesh security |

## References

- ADR 0004 (Platform): process isolation + hardware security
- M3-HARDWARE-INTEGRATION: FAP60 / AntIris / GSM / RFID / TEE access control
- DPA 2019 Kenya §6: biometric data isolation
- Elections Act §34: audit chain / tamper-evident
- NIST SP 800-53 SC-8 (Communications Security), AU-2 (Audit Events)

## M5 Drill Integration

All policies emit `audit_shadow` events with `correlation_key`. Falco logs reference tetragon `process_krn`. Wazuh decodes via `tetragon-decoder.xml`. Cross-system correlation verified at M5 48h drill.
