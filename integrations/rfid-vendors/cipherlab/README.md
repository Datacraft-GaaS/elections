# eel-rfid-cipherlab

CipherLab RS36 handheld RFID adapter for ballot-box cover custody (field staff PDA).

## Hardware

- **Model**: CipherLab RS36 handheld PDA
- **Protocol**: BLE (Bluetooth Low Energy) + USB-serial; CipherLab SDK (commercial)
- **Frequency**: 865-868 MHz (Kenya UHF band per CAK 2022 band plan)
- **Ingress protection**: IP65 (fully dust-tight, water-jet resistant)
- **MDM enrollment**: Android Enterprise, managed via `operations/mdm/`
- **Battery**: 3.7V 4800mAh Li-Ion; target 12h shift life

## Use cases

Field staff open polling-station ballot boxes using this handheld during:
- `STATION_RECEIVED -> STATION_OPENED` transition (dual-officer scan + signature)
- `STATION_OPENED -> VOTING_ACTIVE` clerk EOD-opening event
- `VOTING_ACTIVE -> STATION_CLOSED` closing scan (dual-officer + closing seal photo)
- `STATION_CLOSED -> SEALED_RETURN` egress scan

## Integration

This adapter wraps `services/inventory-svc/py/rfid-reader-driver` via HTTP.
Set `EEL_CIPHERLAB_ADAPTER_URL` to point at the driver instance.

The py driver requires the CipherLab SDK (`pip install cipherlab_sdk`) which is
a commercial dependency. CI uses the `synthetic` adapter.

### Circuit breaker

Threshold: 5 failures in 60s window; 30s cooldown via `eel-resilience-rs`.

## Procurement

- Competitive tender per PPDA regulations.
- CipherLab RS36: ~KES 45,000 per unit; 290 handhelds estimated.
- SDK licence from CipherLab; firmware attestation for SPIRE SVID required.

## Files

- `adapter/handler.rs`: HTTP adapter with circuit breaker.
- `double/mock_cipherlab.rs`: in-memory double; tamper injection + battery-low knob.
- `contract-tests/pact.json`: Pact v2 consumer contract (5 interactions).
