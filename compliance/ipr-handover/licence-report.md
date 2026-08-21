# Third-Party Licence Report

This report summarises SPDX licence identifiers for all third-party dependencies across the eel monorepo. Generated from lock files; regenerate with `just licence-report`.

## Rust (Cargo.lock)

Key dependencies and their licences. Run `cargo-deny check licences` for the full list.

| Dependency | Version (approx) | SPDX Licence |
|---|---|---|
| axum | 0.7.x | MIT |
| tokio | 1.x | MIT |
| serde | 1.x | MIT OR Apache-2.0 |
| serde_json | 1.x | MIT OR Apache-2.0 |
| sqlx | 0.7.x | MIT OR Apache-2.0 |
| reqwest | 0.11.x | MIT OR Apache-2.0 |
| tracing | 0.1.x | MIT |
| thiserror | 1.x | MIT OR Apache-2.0 |
| anyhow | 1.x | MIT OR Apache-2.0 |
| uuid | 1.x | MIT OR Apache-2.0 |
| uuid6 | 0.x | MIT |
| jsonwebtoken | 9.x | MIT |
| opentelemetry | 0.21.x | Apache-2.0 |
| prometheus | 0.13.x | Apache-2.0 |
| cosign | (internal fork) | Apache-2.0 |
| pkcs11 | 0.5.x | Apache-2.0 |
| aes-gcm | 0.10.x | MIT OR Apache-2.0 |
| sha2 | 0.10.x | MIT OR Apache-2.0 |
| diffoscope | (external tool) | GPL-3.0-or-later |

All Rust dependencies are vetted in `deny.toml` under `[licenses]`. GPL-licensed tools (`diffoscope`) are used only in build/verification tooling, not in service binaries.

## Python (uv.lock)

Key dependencies. Run `uv run pip-licences --format=markdown` for the full list.

| Dependency | Version (approx) | SPDX Licence |
|---|---|---|
| Django | 5.x | BSD-3-Clause |
| FastAPI | 0.111.x | MIT |
| pydantic | 2.x | MIT |
| httpx | 0.27.x | BSD-3-Clause |
| uvicorn | 0.30.x | BSD-3-Clause |
| sqlalchemy | 2.x | MIT |
| alembic | 1.x | MIT |
| pytest | 8.x | MIT |
| pytest-asyncio | 0.23.x | Apache-2.0 |
| opentelemetry-sdk | 1.x | Apache-2.0 |
| structlog | 24.x | MIT |
| uuid6 | 0.x | MIT |
| pyright | 1.x | MIT |
| cryptography | 42.x | Apache-2.0 OR BSD-3-Clause |
| python-jose | 3.x | MIT |
| bagit | 1.x | CC0-1.0 |

## Elixir (mix.lock)

| Dependency | Version (approx) | SPDX Licence |
|---|---|---|
| phoenix | 1.7.x | MIT |
| ecto | 3.x | Apache-2.0 |
| postgrex | 0.17.x | Apache-2.0 |
| broadway | 1.x | Apache-2.0 |
| opentelemetry_exporter | 1.x | Apache-2.0 |
| jason | 1.x | Apache-2.0 |
| phoenix_live_view | 0.20.x | MIT |

## Dart (pubspec.lock)

| Dependency | Version (approx) | SPDX Licence |
|---|---|---|
| flutter | SDK | BSD-3-Clause |
| dio | 5.x | MIT |
| riverpod | 2.x | MIT |
| drift | 2.x | MIT |
| isar | 3.x | Apache-2.0 |
| camera | 0.10.x | BSD-3-Clause |
| local_auth | 2.x | BSD-3-Clause |

## Node (pnpm-lock.yaml)

Used for `eel-web-astro` and design tooling only; not included in election-critical service images.

| Dependency | Version (approx) | SPDX Licence |
|---|---|---|
| astro | 4.x | MIT |
| tailwindcss | 3.x | MIT |
| typescript | 5.x | Apache-2.0 |
| vite | 5.x | MIT |

## Licence Compatibility Summary

All dependencies are compatible with the Apache-2.0 licence that governs the custom eel software. GPL-3.0 tools (`diffoscope`) are used only in build verification and do not link with or ship in service binaries.

## Regenerating this report

```sh
just licence-report
```

This runs `cargo-deny check licences`, `uv run pip-licences`, `mix licenses`, and `flutter pub deps` and merges the output into this file.
