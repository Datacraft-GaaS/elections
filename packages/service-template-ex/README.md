# service-template-ex

Mix task for scaffolding Phoenix + LiveView IEBC IEMS microservices.

## Usage

```bash
mix eel.generate ballot_counting
```

Generates:

```
services/ballot_counting/ex/
  mix.exs
  lib/ballot_counting_web.ex
  lib/ballot_counting_web/router.ex
  lib/ballot_counting_web/endpoint.ex
  lib/ballot_counting_web/plugs/auth_plug.ex
  lib/ballot_counting_web/channels/broadway_pipeline.ex
  priv/repo/migrations/001_create_outbox.exs
  priv/repo/migrations/002_create_idempotency.exs
  priv/repo/migrations/003_create_dedup.exs
  deploy/service.yaml
  .woodpecker/ballot_counting.yaml
  docs/README.md
```

## Tests

```
mix test
```
