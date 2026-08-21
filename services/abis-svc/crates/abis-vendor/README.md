# abis-vendor

Vendor SDK abstraction layer. Provides a pluggable, vendor-agnostic facade to biometric matching engines from Neurotechnology, Innovatrics, NEC, Idemia, and others. Runs as a Python FastAPI service on port 8090 listening on Unix socket.

## What it does

- Vendor abstraction: Wraps proprietary SDKs behind a common gRPC interface; vendors can be swapped via environment variable
- Template extraction: Calls vendor SDK to extract biometric template vectors from raw image or signal data
- 1:1 verification: Scores single probe against single candidate template
- 1:N matching: Performs gallery search (candidate list + scores) for single probe
- Threshold tuning: Exposes vendor-specific threshold parameters; captures FMR/FNMR calibration data
- Circuit breaker: Tracks vendor call failures; returns degraded response after 5 consecutive failures; resets on success
- Metrics export: Exposes `/metrics` endpoint (Prometheus format) for call latency, error rate, circuit breaker state

## Public API surface

```rust
pub struct Template {
    pub vendor: String,
    pub data: Vec<u8>,
    pub quality_score: f32,
}

pub struct VerificationResult {
    pub score: f32,
    pub decision: Decision,
    pub latency_ms: u32,
}

pub struct SearchResult {
    pub candidates: Vec<(String, f32)>,
    pub latency_ms: u32,
}

pub async fn extract_template(image: &[u8]) -> Result<Template>;
pub async fn verify_1to1(probe: &[u8], candidate: &[u8]) -> Result<VerificationResult>;
pub async fn search_1ton(probe: &[u8], gallery: &Gallery) -> Result<SearchResult>;
pub fn get_circuit_breaker_status() -> Result<CircuitBreakerState>;
```

## Feature flags

- `neurotec` - Enable Neurotechnology SDK (MegaMatcher, VeriSpeak)
- `innovatrics` - Enable Innovatrics SDK (IDFace, IDLive)
- `nec-neoface` - Enable NEC NeoFace SDK
- `idemia` - Enable Idemia SDK (MorphoMax)
- `test-mock` - Enable mock vendor for testing (always returns 0.5 score)

## Test commands

```bash
# Unit tests (circuit breaker state machine, threshold logic, mock vendor)
cargo test --lib --package eel-abis-vendor

# Integration tests (gRPC server, socket binding, mock SDK calls)
cargo test --test '*' --package eel-abis-vendor -- --test-threads=1

# Property tests (score distribution invariants)
cargo test --package eel-abis-vendor -- --nocapture
```

## Bench commands

```bash
# Template extraction latency (1000 images)
cargo bench --package eel-abis-vendor -- extract_template

# 1:1 verification throughput (baseline scoring)
cargo bench --package eel-abis-vendor -- verify_1to1

# 1:N search latency (10k gallery, probe scoring)
cargo bench --package eel-abis-vendor -- search_1ton

# Circuit breaker overhead (measure state check latency)
cargo bench --package eel-abis-vendor -- circuit_breaker
```

## Dependencies

- `tonic` - gRPC server implementation
- `prost` - Protocol Buffer code generation
- `tokio` - Async runtime, Unix socket binding
- `tracing` - Structured logging for vendor calls
- Vendor SDK dependencies: `neurotec-rs` (or equivalent FFI bindings per vendor)

## Related docs

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Vendor routing diagram, circuit breaker config, fallback strategy
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Vendor SDK license management, circuit breaker manual reset, performance tuning
