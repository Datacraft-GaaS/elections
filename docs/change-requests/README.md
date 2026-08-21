# Change requests

Formal proposals to amend the Datacraft `sw/` response documents where our optimal-choice picks differ from Datacraft-drafted positions. RFP-derived requirements are never subject to change request; only Datacraft's interpretation and framework picks are in scope.

Every CR follows the same shape:
1. **Datacraft position** — what `sw/` currently says.
2. **Our position** — what we recommend.
3. **RFP status** — confirmation that this element is NOT RFP-binding.
4. **Rationale** — evidence-based defence.
5. **Trade-offs** — honest cost of our position.
6. **Fallback** — what we accept if the change is refused.

| CR | Subject | Status |
|---|---|---|
| [001](CR-001-oss-hygiene.md) | OSS-hygiene: OpenBao / Valkey / Karapace vs Vault / Redis / Confluent SR | draft |
| [002](CR-002-portal-stack-non-nextjs.md) | Portal stack Path B vs Next.js 14 | draft |
| [003](CR-003-kenya-sovereign-edge.md) | Kenyan-ISP edge vs Cloudflare / Akamai | draft |
| [005](CR-005-authz-spicedb-vs-cedar.md) | Authz engine: SpiceDB vs Cedar | draft |
| [006](CR-006-mesh-cilium-vs-linkerd.md) | Service mesh: Cilium vs Linkerd 2 | draft |
| [007](CR-007-event-bus-json-vs-avro.md) | Event bus: JSON + per-aggregate topics vs Avro + domain topics | draft |

CR-004 (language tiering) is not raised — user policy holds Rust-default without renegotiation.
