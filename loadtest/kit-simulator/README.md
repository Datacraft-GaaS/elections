# loadtest/kit-simulator

Rust-based simulator for KIEMS election kits. Emulates the full behavioral profile of
65,000 KIEMS kits performing BVI biometric identification, RTS 34A form submission,
polling station check-in, and 30-second heartbeat, under configurable telco impairment.

## Purpose

k6 and Gatling generate plain HTTP load. Kit behavior is more complex: kits maintain
sessions, retry on connectivity loss, queue results locally during offline periods, and
use CBOR wire encoding. The kit simulator replicates this behavioral fidelity, making
it the most realistic load source for election-day rehearsals.

Key behaviors simulated:

- CBOR-encoded BVI and RTS request bodies (matching kit firmware wire format)
- 30-second heartbeat to the fleet management endpoint
- Local result queue with flush-on-reconnect (satellite-path behavior)
- Legacy Android shim path for 22% of kits
- Telco impairment: per-scenario packet loss, latency jitter, satellite-path routing

## Contents

The kit-simulator directory is a Rust workspace crate. Source code lives outside this
directory at `services/biometric-edge-svc/apps/kit-sim/`; this directory contains the
launch scripts and scenario configuration consumed at runtime.

| File | Description |
|------|-------------|
| (binary) `eel-kit-sim` | Compiled kit simulator binary (not committed; build from source) |

Build the binary:

```bash
cargo build --release -p eel-kit-sim
# or via Nix:
nix build .#eel-kit-sim
```

## Usage

```bash
# Run at local scale (1/100) against a local kind cluster
./eel-kit-sim \
  --scenario loadtest/scenarios/election-day.yaml \
  --profile loadtest/profiles/local.yaml \
  --base-url http://localhost:8080 \
  --metrics-port 9090

# Run at staging scale
./eel-kit-sim \
  --scenario loadtest/scenarios/election-day.yaml \
  --profile loadtest/profiles/staging.yaml \
  --base-url https://staging.eel.internal \
  --kit-count 6500

# Satellite fallback mode (Garissa scenario)
./eel-kit-sim \
  --scenario loadtest/scenarios/election-day.yaml \
  --profile loadtest/profiles/local.yaml \
  --base-url http://localhost:8080 \
  --telco-profile satellite \
  --kit-count 100
```

## Config and environment

| Flag | Description |
|------|-------------|
| `--scenario` | Path to scenario YAML |
| `--profile` | Path to scaling profile YAML |
| `--base-url` | Target URL |
| `--kit-count` | Override kit count (default: profile value) |
| `--telco-profile` | `normal`, `degraded`, or `satellite` |
| `--metrics-port` | Prometheus metrics scrape port (default: 9090) |
| `--legacy-pct` | Fraction of kits on legacy Android shim path (default: 0.22) |

## Dependencies

- Rust toolchain (stable, pinned by `rust-toolchain.toml`)
- A running eel deployment at `--base-url`
- Prometheus + Grafana for real-time metrics (optional)

## Test and verify

```bash
# Build
cargo build --release -p eel-kit-sim

# Dry run (no network requests; logs what it would send)
./eel-kit-sim \
  --scenario loadtest/scenarios/election-day.yaml \
  --profile loadtest/profiles/local.yaml \
  --base-url http://localhost:8080 \
  --dry-run \
  --kit-count 10

# Unit tests
cargo test -p eel-kit-sim
```

## Related docs

- `loadtest/README.md`: harness overview
- `loadtest/scenarios/README.md`: scenario format
- `loadtest/profiles/README.md`: scaling profiles
- `services/biometric-edge-svc/apps/kit-sim/`: kit simulator source code

## Limitations

- The simulator does not emulate full Android hardware attestation; the `attest` field in
  requests uses a test credential signed by the test HSM key.
- Very high kit counts (>30k) require a machine with at least 16GB RAM and 8 CPU cores.
- Satellite impairment simulation uses synthetic latency injection, not a real satellite
  link; actual satellite paths have higher and more variable jitter.
