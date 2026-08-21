# eel-observability-py

OTel initialization, SLO helpers, audit shadow triggers, and PII log scrubbing for the eel platform.

## Modules

- `otel` - init_otel, get_tracer, get_meter
- `slo` - record_latency, record_error, record_success, SloTimer
- `audit_shadow` - generate_audit_shadow_trigger
- `log_scrubber` - PiiScrubberFilter, install_scrubber

## Install

```
uv add eel-observability-py
```

## Test

```
uv run pytest tests/ci -x -q
```
