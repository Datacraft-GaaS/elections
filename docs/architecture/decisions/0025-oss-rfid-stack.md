# ADR-0025: Replace Commercial RFID SDKs with Open-Source Equivalents

**Date**: 2026-08-22
**Status**: Accepted

## Context

The RFID ballot-box cover custody system requires drivers for two vendor-specific
reader models:

- Zebra FX9600 fixed UHF reader (warehouse + transit checkpoint gates)
- CipherLab RS36 handheld PDA (field staff at polling stations)

The original adapters (`zebra_link.py`, `cipherlab_rs36.py`) imported commercial
SDKs: `zebralink` and `cipherlab_sdk`.  Neither SDK is distributable under an
open licence.  Procurement timelines for commercial SDK licences are uncertain,
and the licences introduce vendor lock-in that conflicts with the project's
open-source posture and public-audit requirements.

Both pieces of hardware support open, royalty-free protocols as first-class
transport options:

- The Zebra FX9600 implements LLRP (EPCglobal Low Level Reader Protocol 1.1)
  natively, without any ZebraLink dependency.
- The CipherLab RS36 is an Android device and exposes standard BLE GATT profiles
  and Bluetooth SPP.

## Decision

Replace both commercial SDKs with open-source libraries:

| Old | New | Protocol | Licence |
|-----|-----|----------|---------|
| `zebralink` (commercial) | sllurp >= 3.0 | LLRP 1.1 (EPCglobal) | MIT |
| `cipherlab_sdk` (commercial) | bleak >= 0.22 | BLE GATT + SPP | MIT |

Shared implementation details:

1. A new `llrp_common.py` module provides `LlrpCommonClient`, a shared LLRP-1.1
   client used by both `zebra_llrp.py` (replacing `zebra_link.py`) and the
   refactored `impinj_llrp.py`.  All three Impinj + Zebra adapters now share one
   LLRP implementation.

2. `cipherlab_rs36.py` is rewritten to use bleak for BLE scanning, GATT
   characteristic read/write, and BLE Battery Service (UUID 0x180F) for battery
   monitoring.  Bluetooth SPP (UUID 0x1101) is available as a fallback for
   classic-BT-only scenarios.

3. The LLRP client wraps sllurp via `asyncio.get_event_loop().run_in_executor`
   so the asyncio FastAPI driver and the Twisted-based sllurp library coexist
   without event loop conflicts.  Production deployments may opt for
   `twisted.internet.asyncioreactor` for tighter integration.

4. EPC Gen2v2 Authenticate uses ACCESS_SPEC with OpSpec type 1024 as specified
   in EPCglobal EPC Tag Data Standard 2.0 section 8.3.  Gen2v1 Inventory
   fallback is triggered automatically when a tag's `AttributeError` indicates
   no Gen2v2 support.

5. `LlrpCommonClient` contains a built-in 3-state circuit breaker (CLOSED,
   OPEN, HALF_OPEN) with configurable failure threshold and cooldown.

## Consequences

**Positive:**
- No commercial SDK licence procurement required.
- All RFID code distributable under Apache 2.0 / MIT.
- Zebra + Impinj share one LLRP implementation, reducing maintenance surface.
- Tests run in CI without hardware or licence gating (mock sllurp + bleak).
- BLE stack on Android is AOSP (Apache 2.0); no vendor binary required.

**Negative:**
- sllurp uses Twisted; asyncio bridging via thread executor adds overhead.
  Acceptable at Kenya UHF field rates (<= 10 tags/second per reader).
- bleak on macOS + Linux targets the dev harness; production on-device BLE uses
  Flutter platform channels (Android BLE API), a separate code path.
- LLRP sessions require network access to the physical reader IP; CI uses the
  synthetic adapter.

## Alternatives Considered

1. **Negotiate SDK licences**: Rejected.  Timeline risk; licences are not
   distributable to open-source contributors or auditors.

2. **Raw asyncio TCP + LLRP binary framing** (no sllurp): Viable, but
   duplicates message-codec work that sllurp provides.  Chosen to use sllurp
   as the library reference even where Twisted bridging is needed.

3. **Direct Zebra REST API** (port 80/443): Zebra FX9600 exposes a JSON REST
   interface, but it is ZebraLink-specific and still requires the commercial
   stack on the reader side.  LLRP is universal and reader-agnostic.

## References

- sllurp: https://github.com/ransford/sllurp (MIT)
- bleak: https://github.com/hbldh/bleak (MIT)
- LTKC (LLRP Toolkit C): https://sourceforge.net/projects/llrp-toolkit/ (Apache 2.0)
- Android BLE API: https://developer.android.com/guide/topics/connectivity/bluetooth/ble-overview (AOSP, Apache 2.0)
- EPCglobal LLRP 1.1: https://www.gs1.org/standards/epc-rfid/llrp (royalty-free)
- EPC Gen2v2 / ISO 18000-63: https://www.gs1.org/standards/epc-rfid/uhf-rfid-gen2 (royalty-free)
