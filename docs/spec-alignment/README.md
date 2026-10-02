# Spec alignment

The documents under `work_docs/01-Projects/the Commission/sw/` are Datacraft's own earlier drafts. Where they differ from the ADRs in this repo, the ADRs take precedence and the `sw/` documents are to be edited to match. Only the Commission's RFP (`tender.txt`, `PRE_BID_CLARIFICATIONS.md`) is binding; none of the items below are RFP-mandated.

Each note records: what the earlier draft says, the decision per ADR, confirmation that the RFP is silent, the technical rationale (kept for the Commission evaluators and auditors), trade-offs, and the concrete edits to apply to `sw/`.

| note | subject | sw/ sections to edit |
|---|---|---|
| [SA-001](SA-001-oss-hygiene.md) | OpenBao, Valkey, Karapace replace Vault, Redis, Confluent SR | ARCHITECTURE.md §9, §11, §12, §18; SECURITY.md §3.3 |
| [SA-002](SA-002-portal-stack-non-nextjs.md) | Path B portal stack; Next.js excluded | ARCHITECTURE.md §18, §14.1 |
| [SA-003](SA-003-kenya-sovereign-edge.md) | Kenyan multi-ISP edge; no Cloudflare/Akamai | ARCHITECTURE.md §3.1, §10, §12 (align with §17.2) |
| [SA-005](SA-005-authz-spicedb-vs-cedar.md) | SpiceDB for domain authz; OPA/Kyverno for admission | ARCHITECTURE.md §14.4; SECURITY.md §3.1 |
| [SA-006](SA-006-mesh-cilium-vs-linkerd.md) | Cilium CNI + mesh; no Linkerd | ARCHITECTURE.md §6, §14.4, §18; SECURITY.md §2.1 |
| [SA-007](SA-007-event-bus-json-vs-avro.md) | JSON + zstd, per-aggregate topics, 90-day hot retention | ARCHITECTURE.md §9 |

Further `sw/` edits already identified (tracked in `docs/datacraft-internal-backlog.md`): Python 3.12 not 3.13; Rust default with Python fallback; Flutter for kits (revert Kotlin); Ubuntu 24.04; two DR sites; no escrow; FIPS 140-3 L3+ in `expect.md`; on-prem CI (Gitea/Forgejo + Woodpecker) not GitHub/GitLab.
