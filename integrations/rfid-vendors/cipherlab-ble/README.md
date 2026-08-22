# eel-rfid-cipherlab-ble

CipherLab RS36 handheld RFID adapter for ballot-box cover custody (field staff PDA).
Uses BLE + Bluetooth SPP + EPC Gen2 GATT profile via bleak (MIT-licensed).
No commercial CipherLab SDK required.

## Hardware

- **Model**: CipherLab RS36 handheld PDA (Android-based)
- **Protocol**: BLE (Bluetooth Low Energy) + EPC Gen2 GATT profile; SPP fallback
- **Frequency**: 865-868 MHz (Kenya UHF band per CAK 2022 band plan)
- **Ingress protection**: IP65 (fully dust-tight, water-jet resistant)
- **MDM enrollment**: Android Enterprise, managed via `operations/mdm/`
- **Battery**: 3.7 V 4800 mAh Li-Ion; target 12 h shift life

## Open-Source Stack

| Library | Role | Licence |
|---------|------|---------|
| bleak >= 0.22 | Cross-platform Python BLE client | MIT |
| Android BLE API (AOSP) | On-device BLE host stack | Apache 2.0 |
| EPC Gen2 GATT profile | Standard BLE RFID characteristic layout | Open (EPC standard) |
| Bluetooth SPP (UUID 0x1101) | Classic BT serial fallback | Open (Bluetooth SIG) |

bleak is the primary Python library for dev/test BLE access.
Production Flutter integration uses Android platform channels directly
(no bleak dependency on-device).

## Use Cases

Field staff open polling-station ballot boxes using this handheld during:
- `STATION_RECEIVED -> STATION_OPENED` (dual-officer scan + signature)
- `STATION_OPENED -> VOTING_ACTIVE` clerk EOD-opening event
- `VOTING_ACTIVE -> STATION_CLOSED` closing scan (dual-officer + closing seal photo)
- `STATION_CLOSED -> SEALED_RETURN` egress scan

## Integration

This adapter wraps `services/inventory-svc/py/rfid-reader-driver` via HTTP.
Set `EEL_CIPHERLAB_BLE_ADAPTER_URL` to point at the driver instance.

The Python driver selects the BLE adapter via `EEL_RFID_ADAPTER=cipherlab` and
connects to the RS36 at `EEL_CIPHERLAB_ADDR` (BLE MAC address).

### Circuit Breaker

Threshold: 5 failures in 60 s window; 30 s cooldown via `eel-resilience-rs`.

## Procurement

- Competitive tender per PPDA regulations.
- CipherLab RS36: ~KES 45,000 per unit; 290 handhelds estimated.
- No SDK licence required; open BLE APIs used throughout.
- Firmware attestation for SPIRE SVID requires vendor cooperation.

## Files

- `adapter/handler.rs`: HTTP adapter with circuit breaker; reads `EEL_CIPHERLAB_BLE_ADAPTER_URL`.
- `double/mock_cipherlab_ble.rs`: in-memory double; tamper injection + battery-low knob.
- `contract-tests/pact.json`: Pact v2 consumer contract (5 interactions); provider `eel-rfid-cipherlab-ble`.
