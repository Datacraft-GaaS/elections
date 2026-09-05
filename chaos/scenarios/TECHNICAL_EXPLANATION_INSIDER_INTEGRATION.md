# Insider Risk & Integration Failure — Technical Explanation

## Role
Models adversarial and operational-degradation conditions that are not covered by standard infrastructure chaos (pod-kill, network-loss):
- **Integration failure**: external adapter (`adapter/`) becomes unreachable, returns wrong data, or is permanently offline (SMTP, SMS, USSD, IPFS, printer, HSM, WhatsApp, M-PESA, government APIs).
- **Insider risk**: privileged actor (operator, adapter maintainer, CI/CD pipeline, deployment agent) abuses access — data manipulation, audit log deletion, duplicate/malicious events, privilege escalation, supply chain injection (adapter code tampered), HSM key extraction, credential theft, or split-brain vote tampering.

These are the two highest-risk failure modes for EEL because:
1. The election depends on 50+ external integrations for verification, notification, printing, and transparency.
2. A malicious insider with `adapter/` or `hsm-pkcs11/` access can corrupt the audit chain at its source.

## Purpose
Validate that the M3 audit chain (`auditId` UUID7 + `proofHash` SHA-256 + `Ed25519` HSM signature + `correlation_key`) detects and survives both categories. Provide testable scenarios for compliance (DPA 2019 §6, Elections Act §34, ADR-0022/0023) and for security audits.

## Functionality — Integration Failure Scenarios (new files)
- `eel-integration-sms-failure/`: `telco-sms` adapter offline — SMS notifications stop; voter education pipeline degrades; queue builds; no data corruption.
- `eel-integration-usssd-down/`: `telco-ussd` unavailable — USSD voter lookup fails; manual lookup activates; audit records manual override.
- `eel-integration-smtp-down/`: Email/SMTP gateway failure — no report delivery; outbox accumulates; no signed entry lost.
- `eel-integration-ipfs-unavailable/`: IPFS storage unavailable — evidence files queued locally; Merkle root consistent; no object loss.
- `eel-integration-printer-failure/`: `printer-mes` / `gov-printer` unavailable — ballot/badge printing halted; digital ballot backup activates; no physical loss.
- `eel-integration-hsm-timeout-cascade/` (existing 57): HSM unavailable — signing queues; no unsigned entries committed.
- `eel-integration-whatsapp-off/`: WhatsApp Business API down — voter education via WhatsApp stops; SMS fallback activates.
- `eel-integration-kra-access-denied/`: KRA tax/auth integration fails — voter eligibility verification stalls; manual verification activates.
- `eel-integration-mpesa-timeout/`: M-Pesa payment gateway timeout — financial reconciliation stalls; outbox queues; no double-credit.

## Functionality — Insider Risk Scenarios (new files)
- `eel-insider-data-manipulation/`: Adapter (`iprs/` or `kra/`) returns manipulated voter record. System detects mismatch via `correlation_key` and `M3_device_id`; refuses to commit to audit log.
- `eel-insider-audit-log-deletion/`: Privileged user attempts to delete `transparency-svc` audit entries. HSM-signed `auditId` prevents deletion (append-only Merkle tree); alert fires.
- `eel-insider-privilege-escalation/`: Operator escalates privileges in `workforce-svc`. Detection via `falco-rules/` (privilege-escalation rule); `tetragon-policies/` block; `wazuh` alert.
- `eel-insider-duplicate-event-injection/`: Malicious adapter injects duplicate `training.medical.incident.reported` events. Deduplication by `idempotency_key` + `auditId` prevents duplicate; `sigma-rules/` detects replay.
- `eel-insider-supply-chain-adapter/`: Adapter source (`adapter/src/`) modified by CI agent with malicious `hmac_sha256` or `signature` function. `deny.toml` blocks unknown source; `cargo audit` detects; `rust-toolchain.toml` pins compiler.
- `eel-insider-credential-theft/`: `field-mobile-svc` PM card stolen. HSM attestation requires card + biometric; device revocation triggers `crms-svc` alert.
- `eel-insider-vote-manipulation/`: Internal actor attempts to alter `rts-svc` tally. `fhe-tally` and `zk-tally` cryptographic verification detects discrepancy; `audit_shadow` records discrepancy.
- `eel-insider-hsm-key-extraction/`: `hsm-pkcs11/` key extraction attempt. HSM audit log records access; `ed25519-dalek` signature verifies no unauthorized signing.

## Configuration — Integration Failure
- Each integration adapter (`adapter/`) exposes chaos knobs (`double/` mock): latency injection, error rate, timeout, duplicate frames.
- `contract-tests/` verify adapter behavior under failure.
- Scenarios use `HTTPChaos` (delay/abort) for REST adapters; `PodChaos` (kill adapter pod) for full outage; `NetworkChaos` (partition adapter from service) for connectivity loss.
- Recovery: adapter restarts → `double/` mock drains queued events → `outbox` table reaches 0 → `audit_shadow` confirms.

## Configuration — Insider Risk
- Scenarios run in isolated `chaos-testing` namespace; `chaos-mesh.org/v1alpha1` with `auditId` label.
- `falco-rules/eel_siem.yaml` (privilege escalation, unauthorized access, canary token trigger) detects insider actions.
- `wazuh-config/` (operator access, audit replay rules) detects replay/injection.
- `tetragon-policies/` (advanced policy) blocks privilege escalation at kernel level.
- `deny.toml` + `rust-toolchain.toml` prevent supply-chain injection.
- `audit_shadow` + `otel` records every insider action with M5 chain.

## How to run (integration failure)
```bash
kubectl apply -f chaos/scenarios/eel-integration-sms-failure-telco-sms.yaml
# Monitor adapter health: kubectl -n eel-prod get pods -l app=telco-sms
# Verify outbox: SELECT count(*) FROM outbox WHERE processed_at IS NULL;
```

## How to model (insider)
```bash
# Test adapter manipulation: modify adapter/ source; verify cargo deny + audit fails
# Test audit deletion: delete transparency log entry; verify HSM-signed audit prevents
# Test privilege escalation: escalate user; verify falco + tetragon blocks
```

## References
- Integrations directory (`integrations/`): `adapter/` / `double/` / `contract-tests/` structure; ADR-0004/0022/0023; DPA 2019 §6; Elections Act §34
- Security stack (`detection/`): `falco-rules/`, `sigma-rules/`, `tetragon-policies/`, `wazuh-config/`, `canary-tokens/`
- M5 audit chain: `auditId` UUID7 + `proofHash` SHA-256 + `signature` Ed25519 + `correlation_key`
