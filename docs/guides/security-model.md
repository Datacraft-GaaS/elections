# Security Model

## Trust Boundaries

The system has five trust zones. See [cross-cutting/trust-model.md](../architecture/cross-cutting/trust-model.md) for the full diagram.

1. Public (unauthenticated): citizens, media, researchers accessing public APIs via the portal.
2. Edge (WAF-verified): mTLS-verified edge proxies; rate-limiting and DDoS mitigation applied.
3. Internal mesh (SPIFFE mTLS): all service-to-service calls within the cluster.
4. HSM enclave: ABIS biometric operations and key ceremony; no plaintext data ever leaves.
5. External gov systems: IPRS, KRA, EACC, eCourt; accessed via dedicated integration adapters with client certificates.

## PII Tiers

| Tier | Description | Examples | Storage rule |
|---|---|---|---|
| T0 | Biometric | Fingerprint templates, iris scans | ABIS enclave or HSM-encrypted blob; never logged |
| T1 | Personal identity | National ID, name, date of birth, phone | AES-256 at rest; access logged to audit event store |
| T2 | Operational | Station ID, ward, kit serial, poll date | Standard encryption; no per-row access log |
| T3 | Aggregate/public | Turnout counts, published form data, Merkle roots | Unencrypted; public read |

PII classification is enforced at the field level by `eel-pii-{rs,ex,py}` libraries. Any field tagged `#[pii(tier=T1)]` (Rust) or `Annotated[..., PIITier.T1]` (Python) is automatically redacted from logs.

## SPIFFE Identity Model

Every workload runs with a SPIFFE SVID of the form:

```
spiffe://eel.cluster/<namespace>/<service-account>
```

SVIDs are short-lived (1 h) and rotated automatically by SPIRE agents on each node. The SPIRE server is in `platform/spire/`. Loss of the SPIRE server triggers a break-glass procedure: `docs/runbooks/break-glass.md`.

## HSM Roles (ADR-0005)

| HSM partition | Owner | Purpose |
|---|---|---|
| `eel-rts-signing` | rts-svc | Form 34* and Merkle root signing |
| `eel-bvr-cert` | bvr-svc | Voter ID card certificate issuance |
| `eel-training-cert` | training-svc | Poll-worker certificate signing |
| `eel-kit-cert` | inventory-svc | Field kit TLS certificate issuance |
| `eel-abis-session` | abis-svc | Enclave session key derivation (PKCS#11) |
| `eel-transparency` | transparency-svc | Reproducibility bundle signing |

Key ceremonies are documented in `docs/runbooks/hsm-key-ceremony.md`. All ceremony events are witnessed and logged.

## ASVS Compliance

ASVS L3 compliance is enforced for election-critical modules (M02, M03, M06, M08, M10, M16, M21). Automated checks run via `tools/asvs-check`. Results are stored in `compliance/iso-27001/` and `compliance/elections-tech-regs/`. Full checklist: `compliance/ipr-handover/ipr-checklist.md`.

## Secrets Management

Secrets are managed by OpenBao (HashiCorp Vault fork) at `platform/openbao/`. Services access secrets via the SPIFFE-authenticated AppRole or Kubernetes auth method. No secret is embedded in code or container images. The supply-chain pipeline (`tools/supply-chain/`) scans all commits for accidental secret inclusion.

## Network Security

- All external traffic passes through the edge WAF (ADR-0010) with L7 DDoS mitigation.
- Internal cluster traffic is encrypted via the Istio service mesh (mTLS enforced by PeerAuthentication policy).
- Egress to external government APIs is proxied through dedicated integration adapters with IP allowlisting.
- Kit devices communicate over HTTPS/mTLS; certificates are issued per device and revocable within 60 s.
