# eel kit-l1 (M26 - L1 telemetry)

L1 anomaly detection telemetry collector service. Runs on BVR and BVI kit devices alongside or separately to collect sensor telemetry (GPS, accelerometer, battery, network, scan timing) and forward to the anomaly detection backend. Designed as a separate background process for isolation and modularity.

## Purpose

Collects low-level device telemetry to detect anomalous kit device behavior (physical tampering, network anomalies, timing outliers). Does not perform inference; data flows to ai-platform-svc for analysis.

## Features

- GPS location tracking (election station validation)
- Accelerometer data collection (physical tampering detection)
- Battery health and charging events
- Network connectivity and signal strength
- Biometric scan timing metrics
- Local circular buffer (1-hour window, memory efficient)
- Encrypted telemetry batch transmission
- No UI (background service)

## Build

```bash
cd services/ai-platform-svc/apps/kit-l1
flutter build apk --release --flavor kit_l1
```

## Development

```bash
flutter pub get
flutter run --flavor development
```

## Tests

```bash
flutter test
```

## Environment

Configure via `assets/config/`:
- `ANOMALY_API_URL`: telemetry ingest endpoint
- `TELEMETRY_BATCH_SIZE`: events per batch (default: 100)
- `FLUSH_INTERVAL_SECS`: batch flush interval (default: 300)

## Device Integration

Runs on same device as BVR/BVI kit:
- Bundled in MDM profile alongside main kit app
- Separate process for isolation
- Shared device certificate for attestation

## Data Collection

```
GPS -> 1Hz
Accelerometer -> 10Hz
Battery -> 1Hz (state change)
Network -> 1Hz (state change)
Scan timing -> on-event
```

Circular buffer discards oldest data when full. No blocking on network unavailability.

## Architecture

- **Frontend**: Flutter background service
- **Backend**: gRPC client to ai-platform-svc/crates/ai-gateway
- **Storage**: in-memory circular buffer, no persistent queue

## Security

- Telemetry encrypted in transit (TLS)
- No PII collection
- Device attestation via main kit app
- Rate-limited transmission

## Performance

- Memory footprint: < 20MB
- CPU usage: < 2% average
- No impact on main kit app performance

## Related Services

- **ai-platform-svc**: telemetry ingest (ai-gateway crate)
- **anomaly-fraud-detection**: analysis backend
