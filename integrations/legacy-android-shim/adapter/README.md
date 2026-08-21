# legacy-android-shim/adapter

Rust crate wrapping gRPC interface for legacy Android devices.

## Layout

- `src/lib.rs` - public API (ShimService, PayloadUpgrade)
- `src/grpc.rs` - gRPC service implementation (eel.legacy.ShimService)
- `src/upgrade.rs` - schema translation logic (legacy JSON to CloudEvents)
- `src/crypto.rs` - signature chain management (device sig preservation + shim re-sign)

## Key types

PayloadUpgrade: takes legacy JSON, validates device cert, translates schema, returns CloudEvents.

LegacyPayload: kit_id, operation (REG | AUTH | SYNC | VOTE), timestamp, data, device_signature.

CloudEvent: CloudEvents 1.0 format with spec_version="1.0", type="voter.registration" (etc), data (merged legacy data).

## Build

```bash
cargo build -p eel-adapter-legacy-android-shim-rs --release
```

## Environment variables

- ANDROID_GRPC_LISTEN: default 0.0.0.0:9090
- ANDROID_DEVICE_CA_CERT: path to device CA certificate (validates device certs)
- ANDROID_SHIM_KEY: path to shim private key (for re-signing)
- ANDROID_MAX_CONCURRENT: default 5000 (connection limit)

## Testing

```bash
cargo test -p eel-adapter-legacy-android-shim
```

## Device provisioning

Device cert CN = device_id (UUID7). Validated against device-ca.pem at startup.
