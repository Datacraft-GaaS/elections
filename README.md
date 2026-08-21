# eel

Cross-platform 30-module system. Desktop, mobile, and server backend share a common set of feature modules.

## Layout

```
apps/
  desktop/        desktop shell (framework TBD — see ADR-0002)
  mobile/         mobile shell
  server/         backend services
packages/
  shared/         cross-cutting utilities (types, errors, logging)
  sdk/            client SDK consumed by apps/*
  ui/             shared design-system primitives
  config/         shared configs (lint, tsconfig, etc.)
modules/
  mod-01 .. mod-30
    shared/       domain models, contracts, pure logic
    server/       server-side implementation
    desktop/      desktop bindings/screens
    mobile/       mobile bindings/screens
    docs/         module spec + ADRs
infra/
  docker/  k8s/  terraform/
docs/
  architecture/  adr/  research/  api/  modules/  guides/  runbooks/
tests/
  e2e/  integration/
scripts/          repo-wide automation
tools/            developer tooling
assets/           static assets (brand, media)
config/           runtime config templates
```

Per-module code lives under `modules/<name>/<platform>/` so a single team can own a vertical slice end to end. Truly generic code goes in `packages/*`, not `modules/`.

## Getting started

1. Pick the app stack (see `docs/adr/0002-app-stacks.md` — placeholder, to be decided).
2. Rename `modules/mod-01`… once module identities are agreed (see `docs/modules/README.md`).
3. Read `docs/architecture/overview.md` before adding new cross-module dependencies.

## Status

Scaffolding only. No code yet.
