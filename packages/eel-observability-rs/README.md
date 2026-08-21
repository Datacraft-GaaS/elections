# eel-observability-rs

OTel initialization, SLO metric macros, audit shadow triggers, and PII log scrubbing for eel Rust services (ADR-0016).

## Purpose in context

Every eel service must export traces to Tempo, metrics to Prometheus, and structured logs via the tracing subscriber. This crate wires all three in a single `init_otel()` call and provides helpers for recording SLO indicators, installing Postgres audit shadow triggers, and preventing PII from leaking into log output.

## Public modules

| Module | Description |
|---|---|
| `otel` | `init_otel()`, `OtelConfig`, `OtelGuard` |
| `slo` | `SloDefinition`, `slo!` macro |
| `audit_shadow` | `audit_shadow_sql()`, `install_audit_shadow()` |
| `scrubber` | `PiiScrubberLayer` (tracing subscriber layer) |

## Key types

- `OtelConfig` - service name, OTLP endpoint, sampling ratio
- `OtelGuard` - RAII guard; drop at shutdown to flush spans
- `SloDefinition` - `(service, indicator, target_percentile, budget_ms)`
- `PiiScrubberLayer` - tracing layer that redacts fields typed as `PIIRestricted` or `PIISensitive`
- `ObservabilityError` - unified error type

## Layout

```
eel-observability-rs/
  src/
    lib.rs
    otel.rs
    slo.rs
    audit_shadow.rs
    scrubber.rs
    error.rs
  Cargo.toml
```

## Usage

```rust
use eel_observability_rs::{init_otel, OtelConfig, PiiScrubberLayer, install_audit_shadow};

#[tokio::main]
async fn main() {
    let cfg = OtelConfig {
        service_name: "rts-backend".into(),
        otlp_endpoint: "http://otel-collector:4317".into(),
        sample_ratio: 0.1,
    };
    let _guard = init_otel(cfg).unwrap();

    // Audit shadow: mirror mutations from voters to audit.voters_history
    install_audit_shadow(&pool, "voters").await.unwrap();
}
```

## Config / env

| Variable | Description |
|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Overrides `OtelConfig.otlp_endpoint` |
| `OTEL_SERVICE_NAME` | Overrides `OtelConfig.service_name` |

## Dependencies

`opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing`, `tracing-subscriber`, `tracing-opentelemetry`, `serde`, `thiserror`, `tokio`, `sqlx`, `eel-pii-rs`.

## Build

```bash
cargo build -p eel-observability-rs
```

## Test

```bash
cargo nextest run -p eel-observability-rs
```

## Related docs

- `docs/adr/0016-data-classification.md` - audit shadow requirements
- `platform/grafana/` - dashboard definitions that consume SLO metrics
- `packages/eel-pii-rs/README.md` - PII wrappers detected by the scrubber

## Limitations

The scrubber operates on log field names; it cannot redact PII embedded inside opaque string values. Services must use typed PII wrappers rather than bare strings for all personal data fields.
