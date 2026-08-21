# eel-dev kind cluster

Three-node local cluster (1 control-plane + 2 workers) running K8s 1.31 under kind.
CNI: Cilium with kube-proxy replacement disabled (kind-compatible mode).

## Prerequisites

- kind >= 0.25
- kubectl >= 1.31
- helm >= 3.16
- docker or podman socket running
- jq, kcat (kafkacat), psql available on PATH
- `tools/softhsm/init.sh` executed once to set up SoftHSM slots
- `cargo build --release -p hsm-init` executed once

## Quick start

```
just -f environments/dev/kind/justfile create
just -f environments/dev/kind/justfile apply-all
just -f environments/dev/kind/justfile smoke
```

## Recipes

| Recipe | Purpose |
| create | Create cluster and install Cilium CNI |
| destroy | Delete cluster and clean temp volumes |
| load-images | Push locally built eel/* images into kind nodes |
| apply-all | Apply all platform manifests in dependency order |
| smoke | Execute smoke/run.sh end-to-end checks |

## Port mappings

| Host port | Cluster target |
| 8080 | Envoy Gateway HTTP (port 80 inside cluster) |
| 8443 | Envoy Gateway HTTPS (port 443 inside cluster) |

## Volumes

Host paths under `/tmp/eel-dev/` are bind-mounted into nodes for persistent-ish storage across pod restarts (not cluster restarts).

## Scaling factor

`loadtest/profiles/local.yaml` sets `scaling_factor: 0.01`. Every SLA number in the load profiles is multiplied by this factor for local runs.

## SoftHSM setup

Run `bash tools/softhsm/init.sh` once before `apply-all`. This initialises slots for: sth, declaration, kit-ca, credential-ca, code-signing, cycle-kek.
