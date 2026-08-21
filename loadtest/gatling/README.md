# loadtest/gatling

Gatling Scala simulations for high-concurrency eel load scenarios. Gatling complements
k6 for scenarios requiring stateful session modeling or sustained high-concurrency HTTP/2
connections that benefit from Gatling's Netty-based non-blocking I/O.

## Purpose

Gatling is used for:

1. Results flood simulation: 34A forms submitted concurrently from 46,229 polling stations
   in a compressed 2-hour window after 17:00 EAT.
2. Sustained concurrency tests: holding 60,000 open HTTP/2 streams to `rts-svc` for
   10 minutes without connection reuse.
3. Regression baselines: Gatling's HTML reports provide a stable baseline format for
   comparing runs across weeks.

## Contents

| Path | Description |
|------|-------------|
| `build.sbt` | sbt build definition; Gatling plugin version pinned |
| `src/test/scala/eel/` | Simulation classes |
| `src/test/scala/eel/RtsSubmitSimulation.scala` | Results submission flood |
| `src/test/scala/eel/PortalResultsSimulation.scala` | Public results page flood |

## Usage

```bash
# Run the RTS submit simulation
cd loadtest/gatling
sbt "Gatling/testOnly eel.RtsSubmitSimulation" \
  -DEEL_BASE_URL=https://staging.eel.internal \
  -DEEL_KIT_COUNT=6000

# Run all simulations
sbt "Gatling/test" \
  -DEEL_BASE_URL=https://staging.eel.internal

# Output: target/gatling/<simulation-name>-<timestamp>/index.html
```

## Config and environment

| JVM property | Description |
|-------------|-------------|
| `EEL_BASE_URL` | Target base URL (passed as `-DEEL_BASE_URL=...` to sbt) |
| `EEL_KIT_COUNT` | Kit count; simulations use this to set concurrent user counts |

## Dependencies

- JVM 17+
- sbt 1.9+ (managed via sbt-extras or Nix shell)
- Gatling 3.10+ (managed by `build.sbt`)

## Test and verify

```bash
cd loadtest/gatling
# Compile only (fast check)
sbt compile

# Run a minimal simulation (local, 10 users)
sbt "Gatling/testOnly eel.PortalResultsSimulation" \
  -DEEL_BASE_URL=http://localhost:8080 \
  -DEEL_KIT_COUNT=10
# Expected: Gatling report at target/gatling/*/index.html with green OK status
```

## Related docs

- `loadtest/README.md`: harness overview
- `loadtest/k6/README.md`: k6 scripts for the same scenarios
- `loadtest/reports/README.md`: stored report history

## Limitations

- JIT warm-up takes 2-3 minutes; do not interrupt early or the first data will be skewed.
- Gatling does not natively handle CBOR; simulations use JSON bodies like k6.
- sbt download on first run requires internet or a Nexus npm/Maven proxy; use the
  air-gapped-build Nexus for offline runs.
