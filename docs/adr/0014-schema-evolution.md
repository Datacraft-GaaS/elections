# ADR 0014: Schema evolution

Status: accepted
Date: 2026-08-21

## Context

At 69+ services with 15-year retention and a 30-day pre-election software freeze, uncoordinated schema changes are a coordination catastrophe. The policy must be one that any team can follow without a rendezvous.

## Decision

**Expand → migrate → contract as universal policy. No exceptions.**

- Migrations per service, sequentially numbered. **Refinery** (Rust) or `sqlx migrate`; **Django migrations** for T2 admin; **Ecto migrations** for LiveView services.
- **Zero-downtime required**. No `ALTER TABLE` that locks. Use `pg_repack` for rewrites; `CREATE INDEX CONCURRENTLY` always; `NOT VALID` constraints validated in a separate step.
- **Never rename a column**. Add new; dual-write; backfill; migrate readers; drop old in a later release.
- **Every migration ships with a tested rollback**. CI runs `migrate` + `rollback` + `migrate` on every PR.
- **Freeze**: 30 days before election, no schema migrations except CVE-driven; approval requires dual sign-off + tabletop discussion. Enforced by CI gate.
- **CI lint gates**: **atlas** or **squawk** rejects hazardous DDL (locking `ALTER`, unindexed FK, missing rollback).
- **Schema-diff against production baseline**: automated report on every PR; review required for T1 clusters.
- **Event schema evolution** (see also ADR-0015): append-only. New event type versions (`v1`, `v2`) live alongside; consumers handle both. Never rewrite historical events.
- **Cross-service coordination**: not needed if the above rules hold. Services own schemas; contracts are events + OpenAPI, not tables.

## Consequences

Positive: zero-downtime schema changes are routine; rollbacks are tested, not aspirational; freeze window compliance is automated. Schema drift between environments detected in CI, not discovered at deploy.

Negative: expand-migrate-contract requires patience, a rename ships across three or four releases. Cultural discipline required; enforced by lint gates so shortcuts fail the build.

## Alternatives rejected

- Coordinated global migrations: does not scale to 69 services with independent release cadences.
- ORM auto-migrations: not deterministic enough for production.
- Downtime windows for schema changes: incompatible with 24/7 election-period requirements.

## Related

ADR-0012 (which clusters this policy governs), ADR-0015 (event schema evolution mirrors), ADR-0006 (CI infrastructure enforces gates).
