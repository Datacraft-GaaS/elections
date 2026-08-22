# eel-rfid-zebra

Zebra FX9600 fixed reader adapter for ballot-box cover custody.

## Hardware

- **Model**: Zebra FX9600 fixed UHF RFID reader
- **Protocol**: ZebraLink API (HTTP + MQTT); commercial SDK licence required
- **Frequency**: 865-868 MHz (Kenya UHF band per CAK 2022 band plan)
- **Ingress protection**: IP54 (dust and splash resistant)
- **Connectivity**: Ethernet or WiFi; SPIRE SVID via nodeagent at boot
- **Power**: PoE or DC input; solar + battery backup at transit checkpoints

## Integration

This adapter wraps `services/inventory-svc/py/rfid-reader-driver` via HTTP.
Set `EEL_ZEBRA_ADAPTER_URL` to point at the driver instance.

The py driver requires the ZebraLink SDK (`pip install zebralink`) which is
a commercial dependency. CI uses the `synthetic` adapter instead.

### Circuit breaker

Threshold: 5 failures in 60s window; 30s cooldown via `eel-resilience-rs`.

## Procurement

- Competitive tender per PPDA regulations.
- ZebraLink SDK: commercial licence from Zebra Technologies.
- Vendor cooperation required for signed firmware (SPIRE SVID at boot).

## Files

- `adapter/handler.rs`: HTTP adapter with circuit breaker.
- `double/mock_zebra.rs`: in-memory double; tamper injection via `tamper_prob`.
- `contract-tests/pact.json`: Pact v2 consumer contract (5 interactions).
