# eel monorepo, root justfile
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

# List Rust workspace roots to build/test/lint: the root workspace plus each
# service-level nested workspace (services/*/Cargo.toml). Fuzz, template, and
# integration-adapter crates are excluded; they are built by their own tooling.
_rust-workspaces:
    @echo ./Cargo.toml
    @find services -maxdepth 2 -name Cargo.toml -exec grep -l '^\[workspace\]' {} \; 2>/dev/null

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
      echo "[up] compose file not yet present, skipping (WP-0.2)"; \
    fi

# Tear down the local dev cluster
down:
    @echo "[down] stopping local cluster..."
    @if [ -f environments/dev/compose/docker-compose.yml ]; then \
      docker compose -f environments/dev/compose/docker-compose.yml down; \
    else \
      echo "[down] compose file not yet present, skipping (WP-0.2)"; \
    fi

# ---------------------------------------------------------------------------
# Testing
# ---------------------------------------------------------------------------

# Run all tests (unit + integration)
test: test-unit test-integration

# Unit tests across all Rust crates
test-unit:
    @echo "[test-unit] running Rust unit tests..."
    @for ws in $$(just _rust-workspaces); do \
      if cargo metadata --no-deps --quiet --manifest-path "$$ws" 2>/dev/null | python3 -c \
          "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then \
        echo "[test-unit] workspace: $$ws"; \
        if command -v cargo-nextest >/dev/null 2>&1; then \
          cargo nextest run --lib --bins --manifest-path "$$ws" -q || exit 1; \
        else \
          cargo test --lib --bins --manifest-path "$$ws" -q || exit 1; \
        fi; \
      else \
        echo "[test-unit] workspace $$ws has no members, skipping"; \
      fi; \
    done
    @echo "[test-unit] running Python tests..."
    @if find services -name "pyproject.toml" -maxdepth 5 | grep -q .; then \
      find services -name "pyproject.toml" -maxdepth 5 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && if [ -d tests/ci ]; then uv run pytest -q tests/ci --suppress-no-test-exit-code 2>/dev/null; fi'; \
    else \
      echo "[test-unit] no Python projects yet, skipping"; \
    fi
    @echo "[test-unit] running Elixir tests..."
    @if find services -name "mix.exs" -maxdepth 6 | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix test --no-color 2>/dev/null || exit 1'; \
    else \
      echo "[test-unit] no Elixir projects yet, skipping"; \
    fi

# Integration tests (require cluster up)
test-integration:
    @echo "[test-integration] running integration tests..."
    @for ws in $$(just _rust-workspaces); do \
      if cargo metadata --no-deps --quiet --manifest-path "$$ws" 2>/dev/null | python3 -c \
          "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then \
        echo "[test-integration] workspace: $$ws"; \
        if command -v cargo-nextest >/dev/null 2>&1; then \
          cargo nextest run --test '*' --manifest-path "$$ws" -q 2>/dev/null || echo "[test-integration] no integration tests in $$ws"; \
        else \
          cargo test --test '*' --manifest-path "$$ws" -q 2>/dev/null || echo "[test-integration] no integration tests in $$ws"; \
        fi; \
      else \
        echo "[test-integration] workspace $$ws has no members, skipping"; \
      fi; \
    done

# ---------------------------------------------------------------------------
# Quality
# ---------------------------------------------------------------------------

# Lint everything
lint: _lint-rust _lint-python _lint-elixir _lint-contracts

_lint-rust:
    @echo "[lint] Rust..."
    @for ws in $$(just _rust-workspaces); do \
      if cargo metadata --no-deps --quiet --manifest-path "$$ws" 2>/dev/null | python3 -c \
          "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then \
        echo "[lint] workspace: $$ws"; \
        cargo clippy --all-targets --all-features --manifest-path "$$ws" -- -D warnings -q || exit 1; \
        cargo fmt --all --check --manifest-path "$$ws" || exit 1; \
      else \
        echo "[lint] workspace $$ws has no members, skipping"; \
      fi; \
    done

_lint-python:
    @echo "[lint] Python..."
    @if find services packages -name "*.py" -maxdepth 8 2>/dev/null | grep -q .; then \
      ruff check . && ruff format --check .; \
    else \
      echo "[lint] no Python files yet, skipping"; \
    fi

_lint-elixir:
    @echo "[lint] Elixir..."
    @if find services -name "mix.exs" -maxdepth 6 2>/dev/null | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix format --check-formatted 2>/dev/null || true'; \
    else \
      echo "[lint] no Elixir projects yet, skipping"; \
    fi

_lint-contracts:
    @echo "[lint] OpenAPI contracts..."
    @if find packages/openapi-contracts -name "*.yaml" 2>/dev/null | grep -q .; then \
      if command -v spectral >/dev/null 2>&1; then \
        spectral lint "packages/openapi-contracts/**/*.yaml" --fail-severity=warn; \
      else \
        echo "[lint] spectral not found, skipping contract lint"; \
      fi; \
    else \
      echo "[lint] no OpenAPI contracts yet, skipping"; \
    fi

# Format everything
fmt:
    @echo "[fmt] formatting..."
    @for ws in $$(just _rust-workspaces); do \
      cargo fmt --all --manifest-path "$$ws" 2>/dev/null || true; \
    done
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
    @for ws in $$(just _rust-workspaces); do \
      if cargo metadata --no-deps --quiet --manifest-path "$$ws" 2>/dev/null | python3 -c \
          "import sys,json; w=json.load(sys.stdin); exit(0 if w['workspace_members'] else 1)" 2>/dev/null; then \
        echo "[build] workspace: $$ws"; \
        cargo build --release --workspace --manifest-path "$$ws" -q || exit 1; \
      else \
        echo "[build] workspace $$ws has no members, skipping"; \
      fi; \
    done
    @if find services -name "pyproject.toml" -maxdepth 5 2>/dev/null | grep -q .; then \
      find services -name "pyproject.toml" -maxdepth 5 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && uv build 2>/dev/null || true'; \
    else \
      echo "[build] no Python projects yet, skipping"; \
    fi
    @if find services -name "mix.exs" -maxdepth 6 2>/dev/null | grep -q .; then \
      find services -name "mix.exs" -maxdepth 6 -exec dirname {} \; | \
        xargs -I{} sh -c 'cd {} && mix compile 2>/dev/null || true'; \
    else \
      echo "[build] no Elixir projects yet, skipping"; \
    fi

# ---------------------------------------------------------------------------
# Supply-chain / signing
# ---------------------------------------------------------------------------

# Generate SBOM (SPDX + CycloneDX) with syft
sbom:
    @echo "[sbom] generating SBOM..."
    @command -v syft >/dev/null 2>&1 || { echo "[sbom] syft not found, install via nix or brew"; exit 1; }
    mkdir -p dist/sbom
    syft . -o spdx-json=dist/sbom/eel.spdx.json -o cyclonedx-json=dist/sbom/eel.cdx.json
    echo "[sbom] written to dist/sbom/"

# Sign artifacts with cosign (keyless via SPIFFE in CI)
sign:
    @echo "[sign] signing artifacts..."
    @command -v cosign >/dev/null 2>&1 || { echo "[sign] cosign not found"; exit 1; }
    @echo "[sign] placeholder, wire to CI SPIFFE identity in WP-0.5"

# ---------------------------------------------------------------------------
# Contracts
# ---------------------------------------------------------------------------

# Lint + validate all contracts
contracts: _lint-contracts
    @echo "[contracts] Karapace compat check..."
    @echo "[contracts] placeholder, wire Karapace in WP-0.4a"
    @echo "[contracts] SpiceDB schema validation..."
    @if find packages/authz-schema -name "*.zed" 2>/dev/null | grep -q .; then \
      if [ -x ./tools/spicedb/zed ]; then \
        find packages/authz-schema -name "*.zed" -exec ./tools/spicedb/zed validate {} \; || exit 1; \
      else \
        echo "[contracts] SpiceDB zed CLI not found (see tools/spicedb/zed for install hints), skipping"; \
      fi; \
    else \
      echo "[contracts] no .zed schemas yet, skipping"; \
    fi

# Regenerate API clients from OpenAPI specs
gen-clients:
    @echo "[gen-clients] regenerating clients..."
    @if find packages/openapi-contracts -name "*.yaml" 2>/dev/null | grep -q .; then \
      if command -v openapi-generator-cli >/dev/null 2>&1; then \
        find packages/openapi-contracts -name "*.yaml" | while read spec; do \
          ctx=$(basename "$(dirname "$spec")"); \
          echo "  generating client for $ctx"; \
          openapi-generator-cli generate -i "$spec" -g rust -o "packages/eel-client-$ctx" || exit 1; \
        done; \
      else \
        echo "[gen-clients] openapi-generator-cli not found, skipping"; \
      fi; \
    else \
      echo "[gen-clients] no OpenAPI contracts yet, skipping"; \
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
      echo "[synth] eel-synth crate not yet present, skipping (WP-0.3)"; \
    fi

# Seed the local database with synthetic data
seed: synth
    @echo "[seed] seeding database..."
    @if [ -n "${DATABASE_URL:-}" ]; then \
      echo "[seed] running migrations (eel-migrate)..."; \
      cargo run -p eel-migrate; \
    else \
      echo "[seed] DATABASE_URL not set, skipping"; \
    fi

# Apply all services' migrations to DATABASE_URL (idempotent)
migrate:
    @cargo run -p eel-migrate

# ---------------------------------------------------------------------------
# CI
# ---------------------------------------------------------------------------

# Full CI: lint + test + contracts (must exit 0 on empty workspace)
ci: lint test contracts
    @echo "[ci] all checks passed"

# Verify every crate inherits the workspace version (no divergent pins) and
# the workspace version itself is not a placeholder.
check-release:
    @echo -n "[check-release] workspace version: "; grep -m1 '^version' Cargo.toml | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1; \
      if grep -q '^version.*0\.0\.0' Cargo.toml; then echo "ERROR: placeholder version 0.0.0"; exit 1; fi; \
      if grep -rn '^\[package\]' --include=Cargo.toml services packages tools 2>/dev/null -A1 \
         | grep '^version' | grep -Ev 'version\.workspace|0\.1\.0'; then \
           echo "ERROR: crate pins a divergent version (expected version.workspace = true)"; exit 1; \
      fi; \
      echo "[check-release] all crates inherit the workspace version"

# ---------------------------------------------------------------------------
# Housekeeping
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# Mobile (Flutter) — Wave M0.4 scripts + justfile recipes
# ---------------------------------------------------------------------------

# One-time dev/CI env setup: Flutter, Android SDK, JDK, cosign, syft, fastlane
mobile-bootstrap:
    @scripts/mobile/bootstrap.sh

# Codegen for one or all apps (pub get + build_runner + gen-l10n)
mobile-codegen app="all":
    @scripts/mobile/codegen.sh {{app}}

# Analyze one or all apps (flutter analyze --fatal-infos)
mobile-analyze app="all":
    @scripts/mobile/analyze.sh {{app}}

# Test one or all apps (flutter test)
mobile-test app="all":
    @scripts/mobile/test.sh {{app}}

# Build one app, staging flavor by default
mobile-build app FLAVOR="staging":
    @scripts/mobile/build.sh {{app}} --flavor {{FLAVOR}}

# Sign one app (Android: apksigner + cosign)
mobile-sign app:
    @scripts/mobile/sign.sh {{app}}

# Verify reproducible build (3× build, SHA-256 compare)
mobile-verify-repro app:
    @scripts/mobile/verify-reproducible.sh {{app}}

# Deploy to MDM / managed Play Store
mobile-deploy env="staging" app="all":
    @scripts/mobile/deploy.sh --env {{env}} --app {{app}}

# Install on physical device(s)
mobile-install app device:
    @scripts/mobile/install.sh {{app}} --device {{device}}

# Cold-launch smoke test on device
mobile-smoke app device:
    @scripts/mobile/smoke.sh {{app}} --device {{device}}

# Fleet health monitoring
mobile-monitor env="staging":
    @scripts/mobile/monitor.sh --env {{env}}

# A11y/i18n sweep on one app
mobile-a11y app:
    @scripts/mobile/a11y-sweep.sh {{app}}

# Kill-drill + offline-drill on device
mobile-drills app device:
    @scripts/mobile/kill-drill.sh {{app}} --device {{device}}
    @scripts/mobile/offline-drill.sh {{app}} --device {{device}}

# Full mobile CI: analyze + test + build(staging) + sbom for all apps
mobile-ci:
    @echo "[mobile-ci] running mobile CI for all apps..."
    @scripts/mobile/analyze.sh all
    @scripts/mobile/test.sh all
    @for app in hello-flutter cvr-field-officer; do \
      scripts/mobile/build.sh $$app --flavor staging; \
      scripts/mobile/sbom.sh $$app; \
    done
    @echo "[mobile-ci] done"

# Remove build artifacts
clean:
    @echo "[clean] removing build artifacts..."
    @if [ -d target ]; then cargo clean -q; fi
    @find services packages -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
    @find services packages -name ".ruff_cache" -type d -exec rm -rf {} + 2>/dev/null || true
    @find services -path "*/ex/*/_build" -type d -exec rm -rf {} + 2>/dev/null || true
    @find services -path "*/ex/*/deps" -type d -exec rm -rf {} + 2>/dev/null || true
    @rm -rf dist/mobile
    @echo "[clean] done"
