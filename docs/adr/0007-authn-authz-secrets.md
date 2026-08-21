# ADR 0007: Authentication, Authorization, Secrets

Status: accepted
Date: 2026-08-21

## Context

Human authentication, workload identity, domain authorization, admission policy, and secrets management are five distinct concerns. Election-scale requires each in a purpose-built tool with a single choke point per concern. Existing fleet already runs Keycloak, SpiceDB, OpenBao — operational familiarity is not a rounding error over 3 years.

## Decision

- **Human AuthN**: **Keycloak** (Apache 2.0). OIDC + SAML + WebAuthn/FIDO2 + hardware-key. Realms per audience (staff, party, observer, media, agent, citizen). Federated to IPRS / e-Citizen / KRA via identity brokering when those become source of truth (deferred).
- **Workload identity**: **SPIFFE/SPIRE** with HSM-sealed root (ADR-0005). Every workload — pods, kits, mobile apps — carries an SVID (X.509 or JWT) attested from K8s SA + TPM + HSM selectors. mTLS via Cilium mesh anchors on these identities.
- **Domain authorization**: **SpiceDB** (Apache 2.0, Google Zanzibar impl). ReBAC because election permissions are inherently relational (`observer with accreditation A CAN read constituency C after time T`). Canonical `.zed` schema per bounded context in `packages/authz-schema/`, versioned, PR-reviewed, golden-tested.
- **Admission & config policy**: **Kyverno + OPA Gatekeeper** at the K8s admission boundary. SpiceDB decides "can this human/service do this?"; Kyverno/OPA decide "should this K8s action be permitted?" Different concern, different placement.
- **Secrets & PKI**: **OpenBao** (Apache 2.0 fork of Vault, under Linux Foundation, IBM + community). API-compatible with Vault. HSM seal via PKCS#11. Mounts: KV v2, transit, PKI intermediates, database dynamic credentials, SSH CA. HashiCorp Vault rejected (BSL).
- **Portal pattern**: **BFF (backend-for-frontend)** per portal. Browsers never hold refresh tokens, never call SpiceDB or OpenBao. Sessions are HTTP-only + Secure + SameSite=Lax cookies bound to portal origin, backed by server-side session store.
- **Token style**: short-lived JWTs (5-10 min), refresh via silent auth. Admin ops use sender-constrained tokens (DPoP or mTLS-bound).
- **Shared libraries** (one API across languages, enforced at code review): `packages/eel-auth-{rs,py,ex}` (JWT verify + subject extraction + SpiceDB check + audit emit + OTel span decoration) and `packages/eel-secrets-{rs,py,ex}` (OpenBao client with short-lived-cred conventions, HSM-sealed transit). Service code never handles raw tokens or raw secret fetches.
- **Break-glass**: root Keycloak admin, OpenBao root token, SpiceDB admin API key, HSM custodian shares — sealed with Shamir 3-of-5 across the three named custodians. Rotation procedure in `docs/runbooks/break-glass.md`.
- **Audit**: every login (Keycloak), every allow/deny (SpiceDB), every secret read (OpenBao), every SVID issue (SPIRE) → Kafka → ClickHouse (via ADR-0015 event bus).

## Consequences

Positive: single choke point per concern; one library API across languages so no service reinvents auth; existing fleet familiarity; every component Apache 2.0.

Negative: SpiceDB schema becomes a load-bearing artefact across ~100 services — governance required. Keycloak upgrades have historically been painful; version-pinned, rehearsed quarterly. OpenBao is younger than Vault; verify at scale before committing to Vault-only enterprise features.

## Alternatives rejected

Zitadel/Authentik (less battle-tested in gov), OpenFGA (SpiceDB has stronger consistency + tooling), Ory split-stack (more moving parts, no compelling win), Vault (BSL).

## Related

ADR-0004 (SPIRE + Kyverno on K8s), ADR-0005 (HSM roots), ADR-0008 (BFF pattern in portal stack), ADR-0015 (audit event flow).
