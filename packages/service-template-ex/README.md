# service-template-ex

Mix project template for new eel Elixir services: generates a Phoenix/LiveView application pre-wired with all `eel-*-ex` packages and the standard service layout.

## Purpose in context

New Elixir services in `services/<svc>/ex/` are scaffolded from this template using `mix new --template`. The generated project includes a Phoenix router, LiveView surface, Ecto repo, Broadway consumer stub, Oban worker, and the complete `eel-*-ex` dependency set. It ships the conformance test module that verifies ADR-0011 compliance.

## Generated layout

```
<svc>_backend/
  lib/
    <svc>_backend_web/
      router.ex
      live/
    <svc>_backend/
      application.ex
      repo.ex
      domain/
  test/
    ci/               # ExUnit tests discovered by CI
    conformance/      # ADR-0011 conformance suite
  priv/repo/migrations/
  config/
  mix.exs
```

## Usage

```bash
# Scaffold a new Elixir service:
mix new services/my_svc/ex/my_svc_backend \
    --template packages/service-template-ex/template
```

## Conformance suite

```elixir
# test/conformance/conformance_test.exs (generated)
defmodule MyScvBackend.ConformanceTest do
  use ExUnit.Case

  @tag :conformance
  test "api conformance" do
    ServiceTemplateEx.Conformance.run_all("http://localhost:4000")
  end
end
```

## Dependencies

`jason`, `ecto_sql`. Generated project depends on Phoenix, Ecto, Broadway, Oban, and all `eel-*-ex` packages.

## Install

```elixir
{:service_template_ex, path: "../../packages/service-template-ex", only: :dev}
```

```bash
mix deps.get
```

## Test

```bash
mix test
```

## Related docs

- `packages/service-template-rs/README.md` - Rust counterpart
- `packages/service-template-py/README.md` - Python counterpart
- `docs/plans/00-principles.md` - standard Elixir service layout conventions
- `docs/adr/0011-api-conventions.md` - what the conformance suite verifies

## Limitations

The template generates a standard OTP application. Services requiring a custom supervision tree (e.g. biometric sensor GenServers) must add their children manually after scaffolding.
