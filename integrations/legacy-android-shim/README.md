# integrations/legacy-android-shim

I06 Legacy Android APK gRPC shim: upgrades ~14,100 field devices from pre-CloudEvents payload format.

## Purpose

Migration bridge for legacy Huawei/Samsung devices in field. Translates old protobuf schema to current eel CloudEvents v1.0 format. Maintains signature chain: preserves original device signature, adds shim re-signature.

## Modules using this adapter

M01 (bvr-device shim), M05 (bvi-device shim).

## Protocol

gRPC (TLS 1.3, connection pooling for 14k+ devices).

## Authentication

Device certificate per device (x.509 issued by kit-ca). Device identity in CN field of cert.

## PII enforcement

High tier. Biometric templates (fingerprints, iris), voter data. Field-level encryption for storage. No raw payload logging.

## Rate limits

Device-to-service (no per-device throttle). Flow control via gRPC window management. 5k concurrent connections per backend pod.

## Circuit breaker

Open 10s after 5 consecutive failures. Fallback: device queues locally; retry when reconnected.

## Chaos toggle

ANDROID_CHAOS_ENABLED=true (double only). Injects schema error, duplicate frames, signature mismatch.

## Payload upgrade

Legacy JSON -> CloudEvents 1.0. Operation codes: REG, AUTH, SYNC, VOTE mapped to event type. Original timestamp and signatures preserved in spec_version field.

## Device fleet

~14,100 devices: Huawei Y6 (68%), Samsung A10 (22%), misc (10%). Android 8.0+ support.

## Sunset

Planned removal end Wave 3. All new devices use native CloudEvents; legacy fleet decommissioned Q3 2027.

## Related

See adapter/, double/, contract-tests/ subdirectories.
