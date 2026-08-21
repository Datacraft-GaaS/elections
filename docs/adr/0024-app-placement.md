# ADR 0024: App placement inside services

Status: accepted
Date: 2026-08-21

## Context

The scaffold has a `services/<svc>/` layout with `src/`, `tests/`, `migrations/`, `deploy/`, `docs/` per ADR-0020. It does not say where kit firmware (Flutter), mobile apps (Flutter), or operator desktop apps (Tauri) live. During Wave 0 planning the choice matters: it determines where `cargo generate` templates place them, which CI pipelines apply, and whether artifacts land next to the backend they serve or in a shared cross-service tree.

There is also a placement question for `hsm-signer` (P06). ADR-0020 grouped it into transparency-svc rather than making it a 19th service. The rest of the platform-service references (P01/P04/P05/P07/P08/P10) are inferred and require a canonical home.

## Decision

**Client apps live under their home service:**

- Kit firmware and mobile apps: `services/<svc>/apps/<app-name>/` (Flutter/Dart).
- Operator desktop apps: `services/<svc>/apps/<app-name>-desktop/` (Tauri 2, Rust core + web UI).
- Web portals: keep the existing `services/<svc>/{astro,ex,django}/` sub-trees (ADR-0008 + WP-0.1 template).

Rationale: kit and mobile apps consume the API a specific service owns; changes to the API and the client ship together; the same team owns both; test suites are contract-tested against a single backend. Cross-app shared code lives in `packages/kit-sdk-flutter` and `packages/verifier-rs` (see ADR-0002 amendment).

**P06 (HSM signing service):** stays inside `transparency-svc` as crate `hsm-signer` exposing gRPC `Sign/Verify/KeyInfo`. It is not a nineteenth service. The signing service and the transparency log share HSM sessions, key roles, and the ceremony state machine; separating them adds coupling without isolation benefit.

**Other platform services (P01/P04/P05/P07/P08/P10):**

| code | function | home |
|---|---|---|
| P01 | geofence | `services/boundary-svc/` (M18) |
| P04 | IAM (Keycloak, SPIFFE/SPIRE, OpenBao) | `platform/{keycloak,spire,openbao}/` (infra, no service) |
| P05 | transparency log | `services/transparency-svc/crates/{tlog-*}` |
| P06 | HSM signer | `services/transparency-svc/crates/hsm-signer` |
| P07 | change-control board tooling | `services/governance-svc/crates/change-control` |
| P08 | observability | `platform/observability/` (infra) |
| P10 | BI fabric | `services/support-admin-svc/crates/bi-fabric` (Power BI push adapter lives in `integrations/power-bi/`) |

## Consequences

Positive: one place per concern; template generators produce a full-fat service tree with app scaffolds included; no cross-team ownership seams; ArgoCD ApplicationSets fan out per service and include their apps' build artefacts.

Negative: a `services/<svc>/apps/` directory can grow large (up to seven mobile apps in field-mobile-svc). Mitigated by shared `packages/kit-sdk-flutter` for common widgets and `packages/verifier-rs` for shared crypto.

## Alternatives rejected

- Top-level `apps/` tree separate from `services/`: breaks service-team ownership; requires cross-cutting release coordination.
- App-as-service (each Flutter app is its own K8s deployment): apps are client-side artefacts, not K8s workloads; misapplies ADR-0020's topology.
- Nineteenth service for P06 hsm-signer: adds a K8s deployment for a component whose lifecycle is identical to the transparency log.

## Related

ADR-0001 (monorepo), ADR-0002 (languages), ADR-0008 (portal stack), ADR-0017 (transparency log), ADR-0020 (service topology), ADR-0023 (RFP alignment).
