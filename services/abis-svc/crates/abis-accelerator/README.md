# eel-abis-accelerator

GPU/FPGA offload interfaces and CPU fallback for the eel ABIS biometric matcher.

## Architecture

Three matcher implementations behind trait interfaces:

| Accelerator | Throughput | Power | Use Case |
|-------------|-----------|-------|----------|
| GPU (CUDA/ROCm/oneAPI/Metal) | 500k pairs/s | 250W | Large 1:N (>= 10k gallery) |
| FPGA | 1M pairs/s | 75W | Energy-constrained, streaming |
| CPU fallback | 10k pairs/s | 65W | Small gallery, dev, fallback |

## Dispatcher Rules

1. GPU: gallery >= 10k AND latency budget >= 100ms.
2. FPGA: gallery >= 1k AND energy budget specified.
3. CPU: fallback when rules 1 and 2 do not match, or accelerator unhealthy.

## Integration Points

- **GpuMatcher** trait: implement for real CUDA/ROCm/oneAPI/Metal drivers.
- **FpgaMatcher** trait: implement with bitstream management for real FPGA hardware.
- **CpuFallbackMatcher**: always available, uses iterator-based parallelism.
- Consumer: `abis-search-planner` selects an accelerator per shard query.

## Adding a Real GPU Driver

Implement `GpuMatcher` for your hardware driver struct. The dispatcher
will prefer it automatically when gallery size and latency budget match.

## Usage

```rust
use eel_abis_accelerator::dispatcher::select_accelerator;
let selection = select_accelerator(&query, gpu.health(), fpga.health());
```
