.PHONY: reproduce test build

# Primary DoD target: assemble bundle and verify round-trip.
reproduce:
	cargo build --release --bin eel-repro
	cargo test --test bundle_roundtrip -- --nocapture

build:
	cargo build --release

test:
	cargo test
