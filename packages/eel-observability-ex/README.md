# eel-observability-ex

OTel initialization, SLO helpers, audit shadow trigger generation, and PII log scrubbing for eel Elixir services. Mirror of `eel-observability-rs`.

## Purpose in context

Elixir Phoenix services export traces to Tempo, metrics to Prometheus, and structured logs to Loki. This library initializes the OTel SDK in one call, provides `SloTimer` for recording SLO indicators, generates Postgres audit shadow trigger SQL, and installs a Logger filter that redacts PII-typed log metadata.

## Public modules

| Module | Description |
|---|---|
| `Eel.Observability` | `init/1`, `tracer/0`, `meter/0` |
| `Eel.Observability.Slo` | `timer/2`, `record_latency/3`, `record_success/2`, `record_error/3` |
| `Eel.Observability.AuditShadow` | `trigger_sql/1` - generates Postgres audit trigger DDL |
| `Eel.Observability.PiiScrubber` | Logger backend module; redacts PII metadata keys |

## Layout

```
eel-observability-ex/
  lib/
    eel_observability.ex
    eel_observability/
      slo.ex
      audit_shadow.ex
      pii_scrubber.ex
  test/
  mix.exs
```

## Usage

```elixir
# application.ex
def start(_type, _args) do
  Eel.Observability.init(
    service_name: "portal-backend",
    otlp_endpoint: System.get_env("OTEL_EXPORTER_OTLP_ENDPOINT")
  )
  :logger.add_handler(:pii_scrubber, Eel.Observability.PiiScrubber, %{})
  # ...
end

# In a handler:
Eel.Observability.Slo.timer("portal", "page_load", fn ->
  render_page()
end)
```

## Config / env

| Variable | Description |
|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP gRPC endpoint |
| `OTEL_SERVICE_NAME` | Service name for span attributes |

## Dependencies

`opentelemetry_api`, `opentelemetry`, `opentelemetry_exporter`, `jason`.

## Install

```elixir
{:eel_observability, path: "../../packages/eel-observability-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-observability-rs/README.md` - Rust canonical implementation
- `platform/grafana/` - dashboard and alert definitions
- `packages/eel-pii-ex/README.md` - PII types detected by the scrubber

## Limitations

The PII scrubber works on Logger metadata map keys; it cannot redact PII embedded in message strings. Use structured Logger calls with keyword metadata throughout the service.
