# eel-resilience-ex

Elixir resilience patterns for the IEBC IEMS project.

## Features

- **CircuitBreaker** - GenServer with `:closed`, `:open`, `:half_open` states.
- **Retry** - Exponential backoff with optional jitter.
- **Bulkhead** - Poolboy-based concurrency limiter.
- **Timeout** - Task-based function timeout wrapper.
- **IdempotencyPlug** - Idempotency-Key header enforcement for mutating endpoints.
- **DegradationFlag** - Persistent-term-backed degradation mode.

## Usage

Add to your `mix.exs`:

```elixir
{:eel_resilience, path: "packages/eel-resilience-ex"}
```

## Tests

```
mix test
```
