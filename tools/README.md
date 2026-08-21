# tools/

Developer and operator tooling for the eel IEMS. All tools are written in Rust (except where noted) and build from the workspace root with `cargo build --release -p <name>`.

## Tool index

| Name | Language | Purpose | Entry command |
|---|---|---|---|
| `appset-gen` | Rust | Generate ArgoCD ApplicationSets, Kustomize overlays, Cilium NetworkPolicies, and Kafka ACLs from service manifests | `eel-appset-gen` |
| `iam-seed` | Rust | Idempotent IAM bootstrap: Keycloak realms, SpiceDB schemas, SPIRE entries, OpenBao policies | `iam-seed` |
| `hsm-init` | Rust | Bootstrap PKCS#11 keys in SoftHSM slots for local dev and CI | `hsm-init` |
| `hsm-ceremony` | Rust | Multi-custodian HSM key ceremony orchestrator (quorum approval for key rotation) | `hsm-ceremony` |
| `synth` | Rust | Deterministic synthetic data generator (voters, stations, forms, kits, personnel) | `eel-synth` |
| `completeness` | Rust | Stub scanner, traceability mapper, and contract-closure verifier | `eel-completeness` |
| `dr-failover` | Rust | Automated DR failover assist: measures RTO/RPO, plans and executes site failover | `eel-dr-failover` |
| `asvs-check` | Rust | OWASP ASVS 4.0 L2/L3 compliance checker against repo trace tags | `eel-asvs-check` |
| `wcag-audit` | Rust | WCAG 2.2 AA/AAA accessibility auditor using axe-core via headless browser | `eel-wcag-audit` |
| `softhsm` | Bash | SoftHSM2 dev wrapper: slot layout init script | `bash tools/softhsm/init.sh` |
| `spicedb` | Bash | Host-side wrapper for the SpiceDB `zed` CLI | `./tools/spicedb/zed` |
| `kit-simulator` | Rust | KIEMS election-day load simulator: up to 65k virtual BVI and RTS kits | `kit-simulator` |
| `spectral` | JS | OpenAPI Spectral ruleset and custom functions enforcing ADR-0011 | `spectral lint` |

## Build all tools

```bash
cargo build --release
# Individual tool:
cargo build --release -p eel-appset-gen
```

## Common environment variables

| Variable | Used by | Description |
|---|---|---|
| `EEL_HSM_SIGNER_ENDPOINT` | hsm-init, hsm-ceremony | gRPC endpoint for the HSM signer service |
| `PKCS11_MODULE_PATH` | hsm-init, softhsm | Path to `libsofthsm2.so` |
| `SOFTHSM2_CONF` | softhsm | SoftHSM2 config file path |
| `SPICEDB_ZED_BIN` | spicedb wrapper | Override resolved `zed` binary |

## Wave context

| Tool | Wave gate | WP |
|---|---|---|
| softhsm, hsm-init | G0 | WP-0.1 |
| appset-gen, iam-seed, spicedb | G0 | WP-0.3 |
| synth, completeness, spectral | G0 | WP-0.5 |
| hsm-ceremony | G2 | WP-2.2 |
| asvs-check, wcag-audit | G5 | WP-5.12 |
| dr-failover | G5 | WP-5.9 |
| kit-simulator | G3 | WP-3.10 |

## Related docs

- `docs/runbooks/hsm-ceremony.md`: full key-ceremony runbook
- `docs/runbooks/dr-failover.md`: DR failover runbook
- `docs/adr/0011-api-contracts.md`: OpenAPI contract conventions enforced by spectral
- `docs/plans/00-principles.md`: wave and WP overview
