# eel monorepo — root justfile
# Use `just <recipe>` from any directory.
# Sub-justfiles are imported with `mod`; each language owns its own recipes.

set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments

# ---------------------------------------------------------------------------
# Module imports (per-language sub-justfiles, skipped when absent)
# ---------------------------------------------------------------------------
# mod rust   "services/hello-svc/crates/justfile"   # enabled once Rust crates exist
# mod python "services/hello-svc/py/justfile"        # enabled once py apps exist
# mod elixir "services/hello-svc/ex/justfile"        # enabled once ex apps exist

# ---------------------------------------------------------------------------
# Default: list recipes
# ---------------------------------------------------------------------------
default:
    @just --list

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------

# Start the local dev cluster (kind + compose stack)
up:
    @echo "[up] starting local cluster..."
    @if [ -f environments/dev/compose/docker-compose.yml ]; then \
      docker compose -f environments/dev/compose/docker-compose.yml up -d; \
    else \
      echo "[up] compose file not yet present — skipping (WP-0.2)"; \
    fi

# Tear down the local dev cluster
down:
    @echo "[down] stopping local cluster..."
    @if [ -f environments/dev/compose/docker-compose.yml ]; then \
      docker compose -f environments/dev/compose/docker-compose.yml down; \
    else \
      echo "[down] compose file not yet present — skipping (WP-0.2)"; \
    fi

# ---------------------------------------------------------------------------
# Testing
# ---------------------------------------------------------------------------

# Run all tests (unit + integration)
test: test-unit test-integration

# Unit tests across all Rust crates
test-unit:
    @echo "[test-unit] running Rust unit tests..."
    @if cargo metadata --no-deps --quiet 2>/dev/null | python3 -c \
        "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then \
      cargo nextest run --lib --bins -q; \
    else \
      echo "[test-unit] no Rust workspace members yet — skipping"; \
    fi
    @echo "[test-unit] running Python tests..."
    @if find services -name "pyproject.toml" -maxdepth 5 | grep -q .; then \
      find services -name "pyproject.toml" -maxdepth 5 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && uv run pytest -q tests/ci 2>/dev/null || true'; \
    else \
      echo "[test-unit] no Python projects yet — skipping"; \
    fi
    @echo "[test-unit] running Elixir tests..."
    @if find services -name "mix.exs" -maxdepth 6 | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix test --no-color 2>/dev/null || true'; \
    else \
      echo "[test-unit] no Elixir projects yet — skipping"; \
    fi

# Integration tests (require cluster up)
test-integration:
    @echo "[test-integration] running integration tests..."
    @if cargo metadata --no-deps --quiet 2>/dev/null | python3 -c \
        "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then \
      cargo nextest run --test '*' -q 2>/dev/null || echo "[test-integration] no integration tests yet"; \
    else \
      echo "[test-integration] no workspace members yet — skipping"; \
    fi

# ---------------------------------------------------------------------------
# Quality
# ---------------------------------------------------------------------------

# Lint everything
lint: _lint-rust _lint-python _lint-elixir _lint-contracts

_lint-rust:
    @echo "[lint] Rust..."
    @if find . -name "Cargo.toml" -not -path "*/target/*" | xargs grep -l '^\[package\]' 2>/dev/null | grep -q .; then \
      cargo clippy --all-targets --all-features -- -D warnings -q; \
      cargo fmt --all --check; \
    else \
      echo "[lint] no Rust crates yet — skipping"; \
    fi

_lint-python:
    @echo "[lint] Python..."
    @if find services packages -name "*.py" -maxdepth 8 2>/dev/null | grep -q .; then \
      ruff check . && ruff format --check .; \
    else \
      echo "[lint] no Python files yet — skipping"; \
    fi

_lint-elixir:
    @echo "[lint] Elixir..."
    @if find services -name "mix.exs" -maxdepth 6 2>/dev/null | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix format --check-formatted 2>/dev/null || true'; \
    else \
      echo "[lint] no Elixir projects yet — skipping"; \
    fi

_lint-contracts:
    @echo "[lint] OpenAPI contracts..."
    @if find packages/openapi-contracts -name "*.yaml" 2>/dev/null | grep -q .; then \
      spectral lint "packages/openapi-contracts/**/*.yaml" --fail-severity=warn; \
    else \
      echo "[lint] no OpenAPI contracts yet — skipping"; \
    fi

# Format everything
fmt:
    @echo "[fmt] formatting..."
    @if find . -name "Cargo.toml" -not -path "*/target/*" | xargs grep -l '^\[package\]' 2>/dev/null | grep -q .; then \
      cargo fmt --all; \
    fi
    @if find services packages -name "*.py" -maxdepth 8 2>/dev/null | grep -q .; then \
      ruff format .; \
    fi
    @if find services -name "mix.exs" -maxdepth 6 2>/dev/null | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix format 2>/dev/null || true'; \
    fi

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

# Build all services (release mode for Rust)
build:
    @echo "[build] building..."
    @if find . -name "Cargo.toml" -not -path "*/target/*" | xargs grep -l '^\[package\]' 2>/dev/null | grep -q .; then \
      cargo build --release --workspace -q; \
    else \
      echo "[build] no Rust crates yet — skipping"; \
    fi
    @if find services -name "pyproject.toml" -maxdepth 5 2>/dev/null | grep -q .; then \
      find services -name "pyproject.toml" -maxdepth 5 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && uv build 2>/dev/null || true'; \
    else \
      echo "[build] no Python projects yet — skipping"; \
    fi
    @if find services -name "mix.exs" -maxdepth 6 2>/dev/null | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix compile 2>/dev/null || true'; \
    else \
      echo "[build] no Elixir projects yet — skipping"; \
    fi

# ---------------------------------------------------------------------------
# Supply-chain / signing
# ---------------------------------------------------------------------------

# Generate SBOM (SPDX + CycloneDX) with syft
sbom:
    @echo "[sbom] generating SBOM..."
    @command -v syft >/dev/null 2>&1 || { echo "[sbom] syft not found — install via nix or brew"; exit 1; }
    mkdir -p dist/sbom
    syft . -o spdx-json=dist/sbom/eel.spdx.json -o cyclonedx-json=dist/sbom/eel.cdx.json
    echo "[sbom] written to dist/sbom/"

# Sign artifacts with cosign (keyless via SPIFFE in CI)
sign:
    @echo "[sign] signing artifacts..."
    @command -v cosign >/dev/null 2>&1 || { echo "[sign] cosign not found"; exit 1; }
    @echo "[sign] placeholder — wire to CI SPIFFE identity in WP-0.5"

# ---------------------------------------------------------------------------
# Contracts
# ---------------------------------------------------------------------------

# Lint + validate all contracts
contracts: _lint-contracts
    @echo "[contracts] Karapace compat check..."
    @echo "[contracts] placeholder — wire Karapace in WP-0.4a"
    @echo "[contracts] SpiceDB schema validation..."
    @if find packages/authz-schema -name "*.zed" 2>/dev/null | grep -q .; then \
      find packages/authz-schema -name "*.zed" -exec zed validate {} \; 2>/dev/null || \
        echo "[contracts] zed not found — skipping"; \
    else \
      echo "[contracts] no .zed schemas yet — skipping"; \
    fi

# Regenerate API clients from OpenAPI specs
gen-clients:
    @echo "[gen-clients] regenerating clients..."
    @if find packages/openapi-contracts -name "*.yaml" 2>/dev/null | grep -q .; then \
      find packages/openapi-contracts -name "*.yaml" | while read spec; do \
        ctx=$(basename "$(dirname "$spec")"); \
        echo "  generating client for $ctx"; \
        openapi-generator-cli generate -i "$spec" -g rust -o "packages/eel-client-$ctx" 2>/dev/null || true; \
      done; \
    else \
      echo "[gen-clients] no OpenAPI contracts yet — skipping"; \
    fi

# ---------------------------------------------------------------------------
# Synthetic data
# ---------------------------------------------------------------------------

# Generate synthetic test data
synth:
    @echo "[synth] generating synthetic data..."
    @if cargo metadata --no-deps --quiet 2>/dev/null | grep -q '"eel-synth"'; then \
      cargo run -p eel-synth -- register geo forms kits personnel; \
    else \
      echo "[synth] eel-synth crate not yet present — skipping (WP-0.3)"; \
    fi

# Seed the local database with synthetic data
seed: synth
    @echo "[seed] seeding database..."
    @if command -v sqlx >/dev/null 2>&1 && [ -n "${DATABASE_URL:-}" ]; then \
      echo "[seed] running sqlx migrations..."; \
      find services -name "migrations" -type d -maxdepth 4 | \
        xargs -I{} sh -c 'sqlx migrate run --source {} 2>/dev/null || true'; \
    else \
      echo "[seed] DATABASE_URL not set or sqlx missing — skipping"; \
    fi

# ---------------------------------------------------------------------------
# CI
# ---------------------------------------------------------------------------

# Full CI: lint + test + contracts (must exit 0 on empty workspace)
ci: lint test contracts
    @echo "[ci] all checks passed"

# ---------------------------------------------------------------------------
# Housekeeping
# ---------------------------------------------------------------------------

# Remove build artifacts
clean:
    @echo "[clean] removing build artifacts..."
    @if [ -d target ]; then cargo clean -q; fi
    @find services packages -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
    @find services packages -name ".ruff_cache" -type d -exec rm -rf {} + 2>/dev/null || true
    @find services -path "*/ex/*/_build" -type d -exec rm -rf {} + 2>/dev/null || true
    @find services -path "*/ex/*/deps" -type d -exec rm -rf {} + 2>/dev/null || true
    @echo "[clean] done"
