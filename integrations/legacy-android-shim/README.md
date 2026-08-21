# integrations/legacy-android-shim

I06 legacy Android kit shim: upgrades pre-CloudEvents payloads from the
~14,100 legacy Android kits into eel CloudEvents envelopes.

## Wire protocol

`POST /shim/upgrade` accepts legacy JSON payload, returns CloudEvents 1.0
JSON envelope. Original signature preserved in `eel_legacy_sig`. Re-signed
with shim key in `eel_shim_sig`.

## Golden corpus

`contract-tests/fixtures/legacy_payloads.json` contains 20 documented legacy
payload variants covering op codes: REG, AUTH, SYNC, VOTE.

## Fields upgraded

| Legacy field | CloudEvents field |
|---|---|
| kit_id | source suffix + data.kit_id |
| op | type (mapped via _OP_TO_TYPE) |
| ts | time |
| data | data (merged) |
| sig | eel_legacy_sig |

## PII

No voter biometrics in this path. kit_id is operational metadata (Tier D).

## SLA

Sub-millisecond local processing. No upstream calls in normal path.
