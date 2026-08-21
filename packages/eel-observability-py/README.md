# eel-observability-py

OTel initialization, SLO helpers, audit shadow trigger generation, and PII log scrubbing for eel Python services. Mirror of `eel-observability-rs`.

## Purpose in context

Python services export traces to Tempo, metrics to Prometheus, and structured logs to Loki. This package initializes all three in one call and provides helpers for recording SLO indicators, generating Postgres audit shadow trigger SQL, and installing a logging filter that redacts PII-typed fields before they reach the log sink.

## Public API

```python
from eel_observability import (
    init_otel, get_tracer, get_meter,
    record_latency, record_error, record_success, SloTimer,
    generate_audit_shadow_trigger,
    PiiScrubberFilter, install_scrubber,
)
```

| Symbol | Description |
|---|---|
| `init_otel(service_name, otlp_endpoint)` | Initializes OTel SDK; returns a cleanup callable |
| `get_tracer(name)` | Returns an OTel `Tracer` |
| `get_meter(name)` | Returns an OTel `Meter` |
| `SloTimer` | Async context manager; records latency, success, or error on exit |
| `generate_audit_shadow_trigger(table)` | Returns SQL string for a Postgres audit shadow trigger |
| `PiiScrubberFilter` | `logging.Filter` subclass; redacts `PIIRestricted` and `PIISensitive` field values |
| `install_scrubber()` | Installs `PiiScrubberFilter` on the root logger |

## Layout

```
eel-observability-py/
  src/eel_observability/
    __init__.py
    otel.py
    slo.py
    audit_shadow.py
    log_scrubber.py
  tests/
  pyproject.toml
```

## Usage

```python
from eel_observability import init_otel, SloTimer, install_scrubber

cleanup = init_otel("rts-backend", "http://otel-collector:4317")
install_scrubber()

async def handle_tally():
    async with SloTimer("rts", "tally_submit"):
        await do_work()

# On shutdown:
cleanup()
```

## Config / env

| Variable | Description |
|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Overrides `otlp_endpoint` argument |
| `OTEL_SERVICE_NAME` | Overrides `service_name` argument |

## Dependencies

`opentelemetry-api>=1.20`, `opentelemetry-sdk>=1.20`, `opentelemetry-exporter-otlp-proto-grpc>=1.20`.

## Install

```bash
uv add eel-observability-py
```

## Test

```bash
uv run pytest packages/eel-observability-py/tests -v
uv run pyright packages/eel-observability-py/src
```

## Related docs

- `packages/eel-observability-rs/README.md` - Rust canonical implementation
- `platform/grafana/` - dashboard and alert definitions
- `packages/eel-pii-py/README.md` - PII types detected by the scrubber

## Limitations

The PII scrubber works on logging `LogRecord` `extra` dict keys; it cannot redact PII embedded in formatted message strings. Use structured logging with keyword arguments throughout.
