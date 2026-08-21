# kit-simulator

KIEMS election-day load simulator. Drives up to 65,000 virtual BVI (Biometric Voter Identification) and RTS (Results Transmission System) kits against the live cluster, replaying realistic election-day traffic patterns.

Wave context: WP-3.10 (load harness). Required for G3 gate evidence: 46,229 concurrent kit connections at sustained throughput.

## Install

```bash
cargo build --release -p kit-simulator
# Binary at target/release/kit-simulator
```

## Subcommands

| Subcommand | Description |
|---|---|
| `run` | Start the simulator with the given kit count and traffic profile |
| `replay` | Replay a recorded traffic capture from a JSON trace file |
| `validate` | Validate a traffic profile YAML without running |
| `report` | Print a latency/throughput report from a completed run |

## Usage

```bash
# 1000 virtual kits, default profile:
kit-simulator run --kits 1000 --profile election-day

# Full scale (46229 kits, 65000 peak):
kit-simulator run --kits 46229 --profile election-day --duration 8h

# Smoke test (10 kits, 2 minutes):
kit-simulator run --kits 10 --profile smoke --duration 2m

# Replay a recorded trace:
kit-simulator replay --trace loadtest/captures/2026-08-01.json

# Print a report after a run:
kit-simulator report --output loadtest/results/latest.json
```

## Traffic profiles

| Profile | Kits | Concurrent voters | Pattern |
|---|---|---|---|
| `smoke` | 10 | 100 | Uniform ramp |
| `election-day` | 46,229 | 200,000 | Morning surge, midday plateau, evening peak |
| `stress` | 65,000 | 300,000 | Worst-case projection |

Profiles are defined in `loadtest/profiles/`.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `EEL_BVI_ENDPOINT` | `http://localhost:8080` | BVI service gRPC or HTTP endpoint |
| `EEL_RTS_ENDPOINT` | `http://localhost:8081` | RTS results submission endpoint |
| `EEL_KIT_CERT_DIR` | `target/dev-certs/` | Directory with per-kit mTLS certs |
| `EEL_SIM_SEED` | `42` | RNG seed for reproducible runs |
| `EEL_SIM_LOG` | `info` | Log level |

## Output

Results are written to `loadtest/results/` as JSON with per-kit latency percentiles, error rates, and throughput. Use `kit-simulator report` to render a human-readable summary.

## Exit codes

- `0`: all latency SLOs met
- `1`: SLO breach (p99 or error rate exceeded)
- `2`: connection failure or endpoint unreachable

## Related docs

- `services/bvi-svc/README.md`: BVI service
- `services/rts-svc/README.md`: RTS service
- `loadtest/profiles/README.md`: profile format reference
- `docs/runbooks/load-test.md`: pre-election load test runbook
