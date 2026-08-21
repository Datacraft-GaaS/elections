# ADR 0016: PII classification & retention

Status: accepted
Date: 2026-08-21

## Context

Kenya Data Protection Act 2019 §40 grants right-to-erasure. Elections Act §32 statutorily requires 15-year retention of voter register, results, and forms 34A/B. These overlap: most Tier A data most people picture as "PII to erase" is legally exempt from erasure. Where erasure applies (grievance, CMS accounts, helpdesk, non-accredited applications, withdrawn voluntary registrations), soft delete + scheduled purge + rotating backup encryption keys is legally sufficient and simpler than cryptographic erasure. See `docs/architecture/decision-log.md` §3 Phase 12 for the pushback that revised this decision.

## Decision

**Five data-class tiers, enforced in code, not by policy.**

| tier | examples | encryption | access | retention |
|---|---|---|---|---|
| **A — Voter PII (max)** | ID number, biometric templates, biographical data, phone, address, DOB | TDE at rest + column-level for the field where required; ciphertext never leaves DB unless purpose-tagged | JWT + SpiceDB + purpose-of-processing header + audit event | Election cycle + 5y, then anonymised aggregates only |
| **B — Voter operational** | Polling-station assignment, turnout status, queue position | TDE at rest + TLS in transit | Standard authz + audit | Election + 15y (statutory) |
| **C — Public** | Aggregate results, boundary maps, candidate info, gazette, transparency roots | Signature over content (integrity, not confidentiality) | Open | Forever |
| **D — Operational metadata** | Logs, metrics, traces | At-rest encryption | SRE + auditors | 90d hot / 15y cold for election-critical / 30d ops-only |
| **E — Ephemeral** | Sessions, LiveView process state, kit temp state, cache | At-rest, TTL-managed | System-only | 1h - 24h TTL |

**Enforcement (code, not policy)**:

- **Data dictionary per service**: `data_dictionary.yaml` classifying every column. CI fails on any new column without a classification.
- **Rust type wrappers**: `PIIRestricted<T>`, `PIISensitive<T>`, `PIIPublic<T>` — must call `.reveal(purpose: Purpose)` to unwrap; every reveal emits an audit event.
- **Python and Elixir parallels** with equivalent APIs.
- **Logging discipline**: `eel-telemetry` refuses to serialise Tier A/B types; Tier B goes through redaction wrapper.
- **Backup segregation**: Tier A/B backed up with separate keys.

**Right-to-erasure implementation**: **soft-delete + scheduled hard-purge job + rotating backup encryption keys**. NOT cryptographic erasure — over-engineered for this domain.

Rationale (see decision-log §3 Phase 12):
- DPA §40 exceptions cover most Tier A: legal obligation, public interest, legal claims.
- Register mutations, results, forms 34A/B — statutorily retained; refused with lawful basis.
- Grievance intake, CMS accounts, helpdesk, non-accredited media applications, withdrawn VPRS/diaspora registrations — soft-deleted immediately, hard-purged (`DELETE`) after N years per policy per table.
- Nightly logical backups exclude soft-deleted rows past their purge window.
- WAL backups age out at 90 days (much shorter than any erasure window).
- Backup encryption keys rotated annually — old encrypted backups become undecryptable if never restored within the key lifetime.
- Same practical erasure guarantee as per-subject cryptographic erasure, 10× less ops burden.

**Purge job**: runs daily, per service, with an audit log recorded to the audit trail; each purge records (table, count, criteria, timestamp, operator).

## Consequences

Positive: legal-obligation exemptions clean; ancillary systems can honour erasure requests without per-subject DEK management; backup story is coherent (WAL + logical + key rotation); one less ops burden.

Negative: soft-delete rows remain in indices until purge; slight storage/perf overhead until purged. If a regulator later demands per-record cryptographic proof of erasure, we retrofit — accepted risk.

## Alternatives rejected

- Per-subject cryptographic erasure: complex key-management, minimal legal upside for IEBC-specific data landscape.
- Hard delete on request without soft-delete window: destroys review-and-appeal loop; a mistaken request can be irreversible.
- No erasure at all: violates DPA where legal-obligation exemption does not apply.

## Related

ADR-0007 (JWT + SpiceDB + purpose header enforce access), ADR-0013 (event payload PII classification), ADR-0015 (data-class in event envelope for routing), ADR-0012 (backup infrastructure).
