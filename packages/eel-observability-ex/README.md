# eel-observability-ex

OpenTelemetry observability helpers for IEBC IEMS services.

## Features

- **Observability** - Span lifecycle, event recording, error reporting.
- **Slo** - SLO compliance tracking helpers.
- **AuditShadow** - PostgreSQL audit trigger SQL generation.
- **LogScrubber** - Logger backend that redacts PII from log output.

## Usage

```elixir
Eel.Observability.start_span("my.operation", %{user_id: id}, fn ->
  do_work()
end)
```

## Tests

```
mix test
```
