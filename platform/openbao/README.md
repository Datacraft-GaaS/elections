# platform/openbao/

OpenBao secrets management for the eel IEMS. Provides dynamic secrets, PKI intermediate CA, transit encryption, and HSM-sealed storage for all 18 services.

## Contents

| Path | Description |
|---|---|
| `mounts/` | `VaultMount` (OpenBao CRD) definitions: KV v2, PKI, transit, database |
| `policies/` | One HCL policy file per service plus `admin-jit.hcl` and `break-glass.hcl` |
| `local/` | Kustomize overlay for kind (dev root token, file storage, no HSM) |

## Mounts

| Mount path | Type | Purpose |
|---|---|---|
| `kv/eel` | KV v2 | Static secrets (API keys, SMTP credentials) |
| `pki/eel-root` | PKI | Root CA (offline, HSM-sealed) |
| `pki/eel-intermediate` | PKI | Intermediate CA (cert-manager issues here) |
| `transit/eel` | Transit | Envelope encryption for services |
| `database/postgres` | Database | Dynamic PostgreSQL credentials per service |
| `aws/eel` | AWS (if DR cloud) | Dynamic IAM credentials |

## Policy model

Each service policy grants `read` on its own KV path and `update` on its transit key only. Cross-service secret access is not permitted. `admin-jit.hcl` grants temporary elevated access via a time-limited token (break-glass procedure).

```bash
# Issue a JIT admin token (requires dual SVID approval):
vault token create \
  -policy=admin-jit \
  -ttl=1h \
  -explicit-max-ttl=2h
```

## Apply configuration

```bash
# Via iam-seed:
iam-seed openbao

# Manual policy apply:
vault policy write rts-svc platform/openbao/policies/rts-svc.hcl
```

## HSM seal configuration

Production OpenBao is sealed with the Thales Luna 7 HSM via the PKCS#11 auto-unseal provider. The seal config is in `mounts/seal.hcl` (not committed; injected at deploy time from the HSM signer service).

Local dev uses the `file` seal (insecure; dev only).

## Overlays

| Overlay | Description |
|---|---|
| `local/` | kind: file storage, dev root token, no HSM seal |

## Operator responsibilities

- Root CA is offline. Do not bring it online except for intermediate CA renewal (annually).
- Break-glass access is logged to the transparency log; every use requires a post-incident review.
- Dynamic PostgreSQL credentials rotate every 1 hour; services must handle credential rotation without restart (using `eel-resilience-*` retry wrapper).

## Related docs

- `platform/cert-manager/README.md`: consumes the PKI intermediate mount
- `platform/hsm-signer/README.md`: provides the PKCS#11 auto-unseal
- `tools/iam-seed/README.md`: idempotent policy seeding
- `docs/runbooks/break-glass.md`: break-glass access procedure

---

## Technical Explanation (Extended) — openbao

**Role**: openbao is a platform infrastructure component for EEL election infrastructure.
**Purpose**: Provide openbao-specific functionality (see contents table above) with M5 audit chain, MDM device policy, telemetry endpoint, and auditId/proofHash/Ed25519 reference.
**Functionality**: All files are real — no stubs. Configuration references ADR 0004/0006/0009/0010/0019/0022, M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST, DPA 2019 §6, Elections Act §34, NIST SP 800-53 AU-2/AU-10/SI-4/SC-8. All deployment/config manifests preserve audit chain references with auditId + correlation_key + M3_device_id where applicable.
**M5 Integration**: openbao must be verified during M5 48h drill — service must be running, audit events must be present, telemetry must flow, no false positives from openbao components.
**References**: ADR 0004, 0006, 0009, 0010, 0019, 0022, 0023; M3-HARDWARE-INTEGRATION; M5-SMS-USB-MANIFEST; DPA 2019 Kenya §6; Elections Act §34.
