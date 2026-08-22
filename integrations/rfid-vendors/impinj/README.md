# eel-rfid-impinj

Impinj Speedway R700 fixed reader adapter for ballot-box cover custody.

## Hardware

- **Model**: Impinj Speedway R700
- **Protocol**: LLRP (EPCglobal Low Level Reader Protocol) v1.1, royalty-free
- **Frequency**: 865-868 MHz (Kenya UHF band per CAK 2022 band plan)
- **Throughput**: up to 10 tags/second per reader
- **Connectivity**: wired Ethernet (PoE); SPIRE SVID via nodeagent at boot
- **Power**: 100-240V AC with solar + battery backup at transit checkpoints

## Deployment roles

| Role | Count (est.) | Location |
|------|-------------|----------|
| Warehouse gate | 4 | IEBC warehouse, Nairobi |
| Transit checkpoint | 250 | Constituency tally relay points |
| Archive gate | 6 | National archive facility |

## Integration

This adapter wraps `services/inventory-svc/py/rfid-reader-driver` via HTTP.
Set `EEL_IMPINJ_ADAPTER_URL` to point at the driver instance.

### Circuit breaker

Threshold: 5 failures in 60s window; 30s cooldown via `eel-resilience-rs`.

## Procurement

- Competitive tender required per PPDA regulations.
- Estimated unit cost: ~KES 250,000 per fixed reader.
- Approx. 60 fixed gates total.
- Vendor cooperation required for SPIRE-compatible signed firmware.

## Files

- `adapter/handler.rs`: HTTP adapter with circuit breaker.
- `double/mock_impinj.rs`: in-memory double; tamper injection via `tamper_prob` knob.
- `contract-tests/pact.json`: Pact v2 consumer contract (5 interactions).
