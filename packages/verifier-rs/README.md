# verifier-rs

RFC 6962 Merkle inclusion/consistency proof verifier, STH signature check, witness cosign, and bundle hash verification. Exposed as Rust, C ABI (Dart FFI), and WASM.

## Purpose in context

Kit devices (M01, M05, M08, M24) must verify that a voter's registration record or a form-34A tally has been logged to the transparency log before accepting it as authentic. This crate implements the client-side verification algorithm. It compiles to a `cdylib` (`libeel-verifier.so`) consumed by the Flutter SDK via Dart FFI, and to WASM for browser-based verifier portals.

## Public types

| Type | Description |
|---|---|
| `Sth` | RFC 6962 Signed Tree Head (tree size, timestamp, root hash, log key, signature) |
| `InclusionProof` | Sibling hashes + leaf index for a single inclusion proof |
| `ConsistencyProof` | Proof that a newer tree extends an older tree |
| `VerifierError` | Typed error returned by all verify functions |

## Public functions

- `verify_inclusion(sth, proof, leaf_hash) -> Result<(), VerifierError>`
- `verify_consistency(old_sth, new_sth, proof) -> Result<(), VerifierError>`
- `verify_sth_signature(sth) -> Result<(), VerifierError>`
- `verify_bundle_hash(bundle_json, expected_hash) -> Result<(), VerifierError>`

## Layout

```
verifier-rs/
  src/lib.rs        # all types and logic
  build.rs          # cbindgen header generation
  include/
    eel_verifier.h  # generated C header for Dart FFI
  Cargo.toml
```

## Usage (Rust)

```rust
use eel_verifier::{Sth, InclusionProof, verify_inclusion};

let sth: Sth = serde_json::from_str(&sth_json).unwrap();
let proof: InclusionProof = serde_json::from_str(&proof_json).unwrap();
let leaf_hash = eel_crypto_rs::merkle::leaf_hash(&entry_bytes);

verify_inclusion(&sth, &proof, leaf_hash).expect("inclusion proof invalid");
```

## Usage (Dart FFI via kit-sdk-flutter)

```dart
// See packages/kit-sdk-flutter/lib/src/verifier/ for the Dart binding.
final result = EelVerifier.verifyInclusion(sth, proof, leafHash);
if (result != VerifierResult.ok) {
  throw VerificationException(result);
}
```

## Features

| Feature | Description |
|---|---|
| `cdylib` (default) | Builds `libeel-verifier` C ABI |
| `wasm` (default) | Exports `wasm-bindgen` bindings for browser |

## Dependencies

`eel-crypto-rs`, `serde`, `serde_json`, `thiserror`, `hex`. `wasm-bindgen` optional.

## Build

```bash
# Standard Rust
cargo build -p verifier-rs --release

# For Dart FFI (ARM64 Android / x86_64 Linux):
just build-verifier-ffi

# For WASM:
wasm-pack build packages/verifier-rs --target web
```

## Test

```bash
cargo nextest run -p verifier-rs
```

## Related docs

- `services/transparency-svc/docs/EVENTS.md` - tree structure and STH format
- `packages/kit-sdk-flutter/README.md` - FFI binding usage in Flutter
- `docs/adr/0005-cryptography.md` - key roles for log signing

## Limitations

The C ABI uses `cbindgen`-generated headers; regenerate with `just gen-verifier-header` after any public API change. WASM build requires `wasm-pack` and the `wasm32-unknown-unknown` target.
