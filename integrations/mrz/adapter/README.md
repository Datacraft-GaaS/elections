# mrz/adapter

Rust crate wrapping ICAO 9303 MRZ parsing library.

## Layout

- `src/lib.rs` - public API (MrzParser, MrzResult, MrzField enums)
- `src/parser.rs` - checksum validation, TD1/TD2/TD3 format parsing
- `src/fuzz/` - libFuzzer fuzz targets

## Key types

MrzParser: stateless parser with parse(raw_string) -> Result<MrzResult>.

MrzResult: name, document_number, nationality, date_of_birth, expiration_date, sex, optional_fields.

MrzField: enum of parsed data fields with validation state.

## Build

```bash
cargo build -p eel-adapter-mrz-rs --release
```

## FFI for Dart (legacy kit)

Feature flag: `ffi-export`. Builds as cdylib with C ABI for Dart foreign-function calls.

```bash
cargo build --features ffi-export --release
```

## Environment variables

- MRZ_CHAOS_ENABLED: inject parse errors on rare checksums (dev only)

## Testing

```bash
cargo test -p eel-adapter-mrz
cargo fuzz -p eel-adapter-mrz-fuzz
```
