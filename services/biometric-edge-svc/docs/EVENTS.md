# biometric-edge-svc Events

All events are emitted via the edge_outbox table following the eel-outbox-rs protocol.

| Event type | Version | Producer | Consumer | Description |
|---|---|---|---|---|
| kit_heartbeat.v1 | 1 | edge-api /telemetry | monitoring-svc, ecc-svc | Kit liveness + geofence context |
| kit_distress.v1 | 1 | edge-api /distress | ecc-svc | Silent duress alert (EC-IDN-08) |
| kit_ota_delivered.v1 | 1 | edge-api /ota/check | inventory-svc | Confirms kit received OTA manifest |
| kit_bvr_enrolled.v1 | 1 | edge-api /enrolment/upload | bvr-svc, transparency-svc | Edge trace for enrolment initiated at kit |

## Event schema example: kit_heartbeat.v1

```json
{
  "specversion": "1.0",
  "type": "kit_heartbeat.v1",
  "source": "biometric-edge-svc",
  "id": "<uuid7>",
  "time": "<iso8601>",
  "datacontenttype": "application/json",
  "data": {
    "device_serial": "BVRD-KE-00412",
    "station_code": "001-042-003",
    "battery_pct": 82,
    "gps_lat": -1.2921,
    "gps_lon": 36.8219,
    "gnss_fix_quality": 2,
    "uptime_s": 43200,
    "enrolled_count": 48
  }
}
```

## Event schema example: kit_distress.v1

```json
{
  "type": "kit_distress.v1",
  "data": {
    "device_serial": "BVRD-KE-00412",
    "station_code": "001-042-003",
    "timestamp": "<iso8601>",
    "gps_lat": -1.2921,
    "gps_lon": 36.8219,
    "encrypted_context": "<base64-encrypted-blob>"
  }
}
```
