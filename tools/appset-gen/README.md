# eel-appset-gen

ArgoCD ApplicationSet generator for the eel IEMS. Reads service manifests and emits:

- ArgoCD `ApplicationSet` resources (one per service, fanned out per site)
- Kustomize overlay stubs for `primary`, `mirror`, and `edge-*` sites
- Cilium `NetworkPolicy` objects per service
- Kafka ACL YAML fragments

Wave context: WP-0.3 (platform base). Required before any service can be deployed via GitOps.

## Install

```bash
cargo build --release -p eel-appset-gen
# Binary at target/release/eel-appset-gen
```

## Subcommands

| Subcommand | Description |
|---|---|
| `gen` | Generate all output types from service manifests |
| `initial` | Bootstrap initial ApplicationSets from scratch (Wave 0 only) |
| `kafka` | Emit Kafka ACL fragments only |
| `validate` | Dry-run: parse and validate without writing files |

## Usage

```bash
# Generate all output for every service:
eel-appset-gen gen --services services/ --out platform/argocd/applicationsets/generated/

# Generate for a single service:
eel-appset-gen gen --service services/rts-svc --out platform/argocd/applicationsets/generated/

# Dry-run (no writes):
eel-appset-gen validate --services services/

# Kafka ACLs only:
eel-appset-gen kafka --services services/ --out platform/kafka/acls/
```

## Output structure

```
platform/argocd/applicationsets/generated/
  rts-svc.yaml           # ApplicationSet for rts-svc (primary + mirror + edge)
  bvr-svc.yaml
  ...
platform/kafka/acls/
  rts-svc-acls.yaml
```

## Input: service manifest format

Each `services/<svc>/deploy/manifest.yaml` declares:

```yaml
service: rts-svc
sites: [primary, mirror, edge-kisumu, edge-mombasa]
kafka_topics:
  produce: [eel.rts.results.v1]
  consume: [eel.bvr.tallies.v1]
```

`appset-gen` reads these files via the `glob` pattern `services/*/deploy/manifest.yaml`.

## Config / environment

| Variable | Default | Description |
|---|---|---|
| `EEL_ARGOCD_NAMESPACE` | `argocd` | Namespace for generated ApplicationSet resources |
| `EEL_SITES` | `primary,mirror` | Comma-separated list of target sites |
| `EEL_TEMPLATE_DIR` | embedded | Override Tera template directory |

## Exit codes

- `0`: success
- `1`: validation error (missing field, unknown site)
- `2`: I/O error (read or write failure)

## Tests

```bash
cargo test -p eel-appset-gen
```

Snapshot tests live in `tools/appset-gen/src/snapshots/`.

## Related docs

- `platform/argocd/README.md`: ArgoCD install and project structure
- `docs/adr/0019-platform-stack.md` (ADR-0019): platform layer decisions
- `docs/runbooks/gitops-bootstrap.md`: first-time GitOps bootstrap procedure
