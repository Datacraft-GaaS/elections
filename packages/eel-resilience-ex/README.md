# eel-resilience-ex

Circuit breaker, bulkhead, retry, timeout, idempotency, and degradation primitives for eel Elixir services. Mirror of `eel-resilience-rs`.

## Purpose in context

Phoenix services must expose Normal / Degraded / ReadOnlyLockdown modes and protect downstream dependencies with circuit breakers. This library provides Elixir-native implementations of all resilience primitives using GenServers, Tasks, and Plug middleware.

## Public modules

| Module | Description |
|---|---|
| `Eel.Resilience.CircuitBreaker` | GenServer; `call/2`, `state/1`; Closed/Open/HalfOpen |
| `Eel.Resilience.Bulkhead` | Bounded concurrency via `Agent` |
| `Eel.Resilience.Retry` | `with_retry(config, fun)`; jittered exponential |
| `Eel.Resilience.Timeout` | `with_timeout(ms, fun)` |
| `Eel.Resilience.Idempotency` | Plug; deduplicates by `Idempotency-Key` |
| `Eel.Resilience.Degradation` | GenServer flag; `level/1`, `set_level/2` |

## Layout

```
eel-resilience-ex/
  lib/
    eel_resilience/
      circuit_breaker.ex
      bulkhead.ex
      retry.ex
      timeout.ex
      idempotency.ex
      degradation.ex
  test/
  test/support/
  mix.exs
```

## Usage

```elixir
# Supervision tree
children = [
  {Eel.Resilience.CircuitBreaker, name: :abis_cb, threshold: 5, cooldown: 30_000},
  {Eel.Resilience.Degradation, name: :service_mode},
]

# Handler
def handle_biometric(conn) do
  case Eel.Resilience.Degradation.level(:service_mode) do
    :read_only_lockdown -> send_resp(conn, 503, "read-only mode")
    _ ->
      Eel.Resilience.CircuitBreaker.call(:abis_cb, fn ->
        call_abis_service()
      end)
  end
end

# Retry
Eel.Resilience.Retry.with_retry(%{max_attempts: 3, base_ms: 100}, fn ->
  fetch_external_data()
end)
```

## Dependencies

`plug`.

## Install

```elixir
{:eel_resilience, path: "../../packages/eel-resilience-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-resilience-rs/README.md` - Rust canonical implementation
- `docs/adr/0009-resilience.md` - resilience patterns and SLO targets
- `docs/runbooks/<svc>-degradation.md` - per-service degradation runbooks

## Limitations

`Degradation` mode is process-local. Cluster-wide coordination uses a Postgres advisory lock; see the service-specific runbook.
