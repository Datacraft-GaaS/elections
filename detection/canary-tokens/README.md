# Canary Tokens — Honeypot Detection (M5 Hardening)

> Embed non-functional canary tokens in builds and on-device. Access of any token = immediate alert routed to falco, wazuh, audit_service, tetragon, and SRE pager (EMERGENCY only).

## What

Canary tokens are **non-functional decoys** embedded during build (`build.rs` / `build.py` / `build.dart`). They look legitimate but contain no real data. Any access by an unexpected process, network request, or audit query is treated as a **security incident** with high confidence.

## Files

| File | Purpose | Tokens |
|---|---|---|
| `honeytokens.yaml` | Original 4 tokens (HSM key, audit record, biometric template, IPRS record) | 4 |
| `distributed_canary.yaml` | 17 tokens across 6 namespaces | 17 |
| `deployment_notes.md` | Build-time injection + runtime verification + alert routing | — |
| `m5_drill_playbook.md` | 6-phase 48h drill playbook | — |
| `correlation_routing.md` | Maps every token to falco/wazuh/audit/tetragon rules | 17 |

**Total: 17 distributed canary tokens + full drill + routing.**

## Design Principles

- **Non-functional**: no valid biometric data, no real keys, no real credentials
- **High-confidence alert**: access by unexpected process = breach with high certainty
- **Multi-layer**: every token triggers falco + wazuh + audit_service + tetragon simultaneously
- **SRE escalation**: EMERGENCY tokens (HSM key, supply chain, root) trigger PagerDuty within 30s
- **Correlation key**: every token has unique `correlation_key` matching `correlation_routing.md`
- **Audit format**: every token has `auditId` + `proofHash` + `Ed25519 signature` reference
- **M3 device reference**: hardware tokens include `M3_device_id` for device attribution

## Token Taxonomy (17 tokens across 6 namespaces)

### Audit Chain (3 tokens)
Fake audit entries mimicking real voter registration, tally submission, form 34A signed. Triggered when audit service query returns these IDs from a non-attestation process. Detects **audit replay / tampering**.

### Biometric (4 tokens)
Fake fingerprint templates (ANSI-378, ISO-19794-2), iris template (ISO-19794-6), MRZ scan (ICAO-9303). Stored in BVR/BVI outbox. Triggered on unauthorized read/write. Detects **biometric template exfiltration / replay**.

### Crypto / HSM (3 tokens)
Fake Ed25519 signing key, AES-256-GCM blob, JWT bearer token. Triggered on read/extract/decrypt. Detects **key extraction / decryption attack**. **EMERGENCY** severity → SRE pager.

### Network / C2 (3 tokens)
Fake BFF endpoint URL, fake IPRS national ID, fake API credential. Triggered on any network request. Detects **C2 / exfiltration / credential reuse**.

### Supply Chain (2 tokens)
Fake native library (`.so`), fake APK with SHA-256. Triggered on load/replace/signature mismatch. **EMERGENCY** severity → SRE pager. Detects **repackaging / build injection**.

### MDM / Device Policy (2 tokens)
Fake kiosk exit signal, fake admin policy file. Triggered on MDM signal/policy modification. Detects **MDM bypass / kiosk violation**.

## Severity & Escalation

| Severity | Tokens | SRE Pager | Response Time |
|---|---|---|---|
| EMERGENCY | crypto-corr-ed25519-001, sc-corr-library-001, sc-corr-apk-001 | **MUST** | ≤ 30s |
| CRITICAL | Audit chain (3), Biometric (4), Crypto (2), Network (3), Supply (1), MDM (1) | optional | ≤ 60s |
| WARNING | bio-corr-mrz-001 | optional | ≤ 60s |

## Correlation Routing

Every token maps to:
- **Falco rule(s)**: see `correlation_routing.md`
- **Wazuh rule ID(s)**: see `correlation_routing.md`
- **audit_service event(s)**: event type for token type
- **Tetragon policy**: file/process/network policy

Example (crypto-corr-ed25519-001):
```
canary-crypto-ed25519-001
→ falco: hsm_access_unexpected, ed25519_key_access, keystore_unexpected
→ wazuh: 100100, 100101, 100102, 100400
→ audit: crypto_access, key_access, ed25519_sign_attempt
→ tetragon: crypto_file_access, crypto_process_access, file_integrity
→ SRE: PAGER (EMERGENCY)
```

## M5 Drill

All 17 tokens must be verified at M5 48h drill:
- Phase 1: token presence check (build-time injection verified)
- Phase 2: token activation (simulated attacks on all 17)
- Phase 3-4: MDM / supply chain specific tokens
- Phase 5: full correlation verification (all 4 systems)
- Phase 6: M5 drill report with audit reference format

See `m5_drill_playbook.md` for detailed drill steps and `05_dr_report.md` in `../playbooks/` for report template.

## References

- ADR 0004 (Platform): process isolation + hardware security
- ADR 0006 (Build & supply chain): build injection detection
- ADR 0010 (Edge & DDoS): network exfil / C2 detection
- ADR 0019 (Edge topology): GPS/station polygon, BFF endpoint
- ADR 0022 (Release & DR): M5 48h drill, sequential rules
- ADR 0023 (RFP alignment): MDM rollout, audit chain
- DPA 2019 Kenya §6: biometric data, HSM keys
- Elections Act §34: tamper-evident audit, form integrity
- M3-HARDWARE-INTEGRATION: M3 device paths / hardware canaries
- M3-SMS-USB-MANIFEST: GSM / USB canaries

## Extending Canary Tokens

1. Add token to `distributed_canary.yaml` with new `correlation_key`
2. Add entry to `correlation_routing.md` (falco rule / wazuh rule / audit event / tetragon policy)
3. Add wazuh rule to `../wazuh-config/rules/eel_extended_rules.xml` with correlationKey field
4. Add falco rule to `../falco-rules/eel_extended.yaml` with same correlation_key in output
5. Test via `../playbooks/01_canary_activation.md`
6. Update `m5_drill_playbook.md` with new token phase
