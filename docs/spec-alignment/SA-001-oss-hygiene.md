# SA-001: OSS-hygiene consolidation — OpenBao, Valkey, Karapace

Date: 2026-08-21
Applies to: Datacraft bid documents under work_docs/01-Projects/IEBC/sw/ (our own earlier draft)
Status: decided (ADR takes precedence; sw/ docs to be edited)

## Earlier draft (sw/) says

`sw/ARCHITECTURE.md` §18 tech-choices table pins:

- **HashiCorp Vault** for secrets and PKI.
- **Redis 7 Cluster** for cache + Pub/Sub + rate-limiting.
- **Confluent Schema Registry** with Avro for Kafka schemas.

## Decision (per ADRs)

Substitute drop-in OSS-clean forks:

- **OpenBao** (Apache 2.0, Linux Foundation, IBM + community) instead of Vault. Vault API-compatible.
- **Valkey 8** (Linux Foundation BSD-3, Redis-fork) instead of Redis 7+. Wire-protocol identical.
- **Karapace** (Aiven Apache 2.0 fork of Confluent Schema Registry) instead of CSR. API-identical.

## RFP status

**Not RFP-binding.** `tender.txt` does not name any specific secrets store, cache technology, or schema registry. Confirmed by grep of the full tender text.

## Rationale

1. **Licence hygiene**. Vault (BSL since Aug 2023), Redis (7.4+ SSPL/RSAL since Mar 2024), and Confluent CSR (Community License) are **source-available, not OSS**. All three violate the "OSS-first, no licensing/integration costs" principle the user has committed to.
2. **Regulatory audit exposure**. In a 3-year contract with 15-year audit retention, licence changes by the upstream vendor are a supply-chain risk. All three have already changed their licence terms once; the direction is not reversing.
3. **Client-fleet operational familiarity**. The client's existing infrastructure (per `~/.claude/CLAUDE.md` fleet inventory) already runs OpenBao at `62.169.25.77` and MinIO. Operational muscle exists.
4. **Drop-in compatibility**. Each substitute is API-wire-compatible with what it replaces. No re-architecture; only re-pin.
5. **Auditor-defensibility**. On a national-election contract, "Apache 2.0 + Linux Foundation" reads better than "BSL, may be relicensed by vendor" to security auditors.

## Trade-offs

- OpenBao is younger than Vault; some Vault Enterprise features (namespaces, performance replication modes) not yet mirrored. **We do not need Vault Enterprise features** — every capability we use (KV, PKI, transit, database dynamic creds, HSM seal via PKCS#11) is in OpenBao today.
- Valkey is a straight fork; identical semantics.
- Karapace supports both Avro and JSON Schema; if Datacraft has existing Avro artefacts, they load unchanged.

## Edits to apply to sw/

If Datacraft mandates the source-available originals, we accept **only** on documented commitment to (a) fund the licence-terms-change risk over the 15-year retention window, and (b) accept the auditor-response burden of defending non-OSS in the security-critical path.

## Related

ADR-0003 (data infra), ADR-0007 (authn/authz/secrets), ADR-0015 (event bus), ADR-0023 (RFP-alignment).
