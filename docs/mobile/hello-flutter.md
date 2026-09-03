# hello-flutter (CI validator) — Maintenance Plan

Purpose: `hello-svc` reference scaffold; validates kit-sdk wiring in CI.
Intentionally minimal — do NOT grow features.

## Work items

1. Consume eel-core bootstrap (proves the foundation package in the hello
   harness — required by Wave M0 exit).
2. Exercise: attest (stub, dev flavor), outbox round-trip against hello-api,
   telemetry session to telemetry-ingest dev double.
3. Keep one widget test + one integration smoke; used as first target of
   `just mobile-ci` per master §11 Wave M0 exit.

## Definition of done

Serves as canary: `just mobile-ci` runs it first; any kit-sdk/eel-core
regression breaks here first, not in production apps.