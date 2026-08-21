# SA-006: Service mesh — Cilium (CNI + mesh + policy + observability) vs Linkerd 2

Date: 2026-08-21
Applies to: Datacraft bid documents under work_docs/01-Projects/IEBC/sw/ (our own earlier draft)
Status: decided (ADR takes precedence; sw/ docs to be edited)

## Earlier draft (sw/) says

`sw/ARCHITECTURE.md` §6, §14.4 and §18 pin **Linkerd 2** as the service mesh for mTLS enforcement.

## Decision (per ADRs)

**Cilium** (Isovalent, Apache 2.0) as the unified CNI + service mesh + NetworkPolicy enforcement + observability (via Hubble) plane. Linkerd not used.

## RFP status

**Not RFP-binding.** tender.txt does not name any specific service mesh. Confirmed by grep.

## Rationale

1. **Consolidation**. In a Cilium-only world, one operator + one CRD family covers CNI, mesh, NetworkPolicy, L7 filtering, and observability. In a Linkerd + separate-CNI world (Calico/Flannel), we run two operator lifecycles, two CRD families, two upgrade cadences, two failure modes. At 18 K8s services (ADR-0020) across 2 DR sites + 5 edges = 7 clusters, the operational compounding matters.
2. **eBPF-native performance**. Cilium's data plane runs in the kernel via eBPF; Linkerd uses a per-pod sidecar proxy (Linkerd2-proxy in Rust). Cilium eliminates the sidecar hop and its resource overhead. On dense pod deployments (LiveView pool with millions of connections, ADR-0008), sidecar overhead is a real cost.
3. **Hubble observability**. Cilium's Hubble gives per-flow observability out of the box (identity-aware, L7-aware) without a separate deployment. Matches ADR-0004's OpenTelemetry + Grafana + Loki + Tempo integration natively.
4. **Kernel security path**. Cilium can enforce NetworkPolicy at kernel level (tetragon) with L7 awareness, giving us a native path to enforce "service A can talk to service B on exactly method X" without a separate NetworkPolicy operator.
5. **SPIFFE integration**. Cilium 1.14+ integrates directly with SPIFFE/SPIRE (ADR-0007), matching our identity model natively.

## Trade-offs

- Cilium is a bigger technology (larger surface, more concepts). Learning cost is real; documented, active community; mitigated by "one thing to learn" beating "three things to learn."
- Cilium's Service Mesh mode (1.14+) is younger than Linkerd's mesh — Linkerd is more prescriptive and mature-in-the-narrow-sense. Trade-off: Cilium is broader-and-younger; Linkerd is narrower-and-older. For a 3-year contract we pin conservative versions and rehearse upgrades.
- Some ecosystem tools (Linkerd Buoyant tooling, some observability plugins) are Linkerd-native. Cilium equivalents exist for all our needs.

## Edits to apply to sw/

If Linkerd is mandated on operational-familiarity grounds (Datacraft ops team already trained on Linkerd): we accept **Linkerd 2 + Cilium as CNI-only** (Cilium in CNI-only mode; Linkerd for mesh). This gets us most of the eBPF network-policy and observability wins without forcing the ops team off Linkerd. Loses the consolidation win but keeps the operational muscle.

## Related

ADR-0004 (platform), ADR-0007 (SPIFFE integration), ADR-0023 (RFP-alignment).
