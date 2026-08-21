# service-template-rs

`cargo-generate` template for new eel Rust services; also ships the shared conformance test harness.

## Purpose in context

Scaffolds a new service in the correct workspace layout with all `eel-*-rs` crates pre-wired. The template generates the four standard crates (`<svc>-api`, `<svc>-domain`, `<svc>-projections`, `<svc>-worker`), migrations directory, deploy Kustomize base, loadtest profile, and TRACEABILITY.yaml. The `conformance-suite` feature provides tests that any service can import to verify it meets ADR-0011 requirements.

## Public modules

| Module | Feature | Description |
|---|---|---|
| `conformance` | `conformance-suite` | Health, readiness, metrics, problem+json, Idempotency-Key, signing header tests |

## Layout

```
service-template-rs/
  src/
    lib.rs                # conformance harness (feature-gated)
    conformance/
      health.rs
      problem.rs
      idempotency.rs
      signing.rs
  template/               # cargo-generate template files
    {{project-name}}-api/
    {{project-name}}-domain/
    {{project-name}}-projections/
    {{project-name}}-worker/
    Cargo.toml.liquid
    cargo-generate.toml
  Cargo.toml
```

## Usage

```bash
# Scaffold a new service:
cargo generate --path packages/service-template-rs/template \
    --name my-svc \
    --destination services/
```

## Conformance tests (from a service's test suite)

```rust
#[cfg(test)]
mod conformance {
    use service_template_rs::conformance::run_all;

    #[tokio::test]
    async fn api_conformance() {
        run_all("http://localhost:8080").await;
    }
}
```

## Dependencies

`tokio`, `serde`, `serde_json`, `thiserror`, `tracing`. `reqwest` optional via `conformance-suite`.

## Build

```bash
cargo build -p service-template-rs
cargo build -p service-template-rs --features conformance-suite
```

## Test

```bash
cargo nextest run -p service-template-rs
```

## Related docs

- `docs/adr/0011-api-conventions.md` - what the conformance suite verifies
- `docs/plans/00-principles.md` - standard service layout
- `packages/eel-api-server-rs/README.md` - the server builder every generated service uses

## Limitations

The template generates a workspace member; run `cargo build --workspace` from the repo root after scaffolding to verify the new crates compile. The conformance suite requires the target service to be running and reachable at the supplied URL.
