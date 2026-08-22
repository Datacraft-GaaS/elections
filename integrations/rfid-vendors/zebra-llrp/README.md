# eel-rfid-zebra-llrp

Zebra FX9600 fixed reader adapter for ballot-box cover custody.
Uses LLRP (EPCglobal Low Level Reader Protocol 1.1, royalty-free open standard).
No commercial SDK required.

## Hardware

- **Model**: Zebra FX9600 fixed UHF RFID reader
- **Protocol**: LLRP 1.1 (EPCglobal, royalty-free open standard, natively supported by the FX9600)
- **Frequency**: 865-868 MHz (Kenya UHF band per CAK 2022 band plan)
- **Ingress protection**: IP54 (dust and splash resistant)
- **Connectivity**: Ethernet or WiFi; SPIRE SVID via nodeagent at boot
- **Power**: PoE or DC input; solar + battery backup at transit checkpoints

## Open-Source Stack

| Library | Role | Licence |
|---------|------|---------|
| sllurp >= 3.0 | Python LLRP client (Simple LLRP) | MIT |
| LTKC (LLRP Toolkit C) | C reference implementation of LLRP message codec | Apache 2.0 |
| EPCglobal LLRP 1.1 spec | Protocol definition | Royalty-free |

sllurp is the primary Python library.  It wraps the LLRP binary framing
(CONNECT, SET_READER_CONFIG, ADD_ROSPEC, ENABLE_ROSPEC, START_ROSPEC,
RO_ACCESS_REPORT) and is installed as a normal pip dependency with no
licence gating.

## Integration

This adapter wraps `services/inventory-svc/py/rfid-reader-driver` via HTTP.
Set `EEL_ZEBRA_LLRP_ADAPTER_URL` to point at the driver instance.

The Python driver selects the LLRP adapter via `EEL_RFID_ADAPTER=zebra` and
connects to the physical reader at `EEL_ZEBRA_HOST`.

### Circuit breaker

Threshold: 5 failures in 60 s window; 30 s cooldown via `eel-resilience-rs`.

## Procurement

- Competitive tender per PPDA regulations.
- Vendor cooperation required for signed firmware (SPIRE SVID at boot).
- No SDK licence required; LLRP is an open, royalty-free standard.

## Files

- `adapter/handler.rs`: HTTP adapter with circuit breaker; reads `EEL_ZEBRA_LLRP_ADAPTER_URL`.
- `double/mock_zebra_llrp.rs`: in-memory double; tamper injection via `tamper_prob`.
- `contract-tests/pact.json`: Pact v2 consumer contract (5 interactions); provider `eel-rfid-zebra-llrp`.
