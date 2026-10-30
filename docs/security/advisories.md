# Dependency advisory triage

Last reviewed: 2026-10-30. Gate: `cargo deny check advisories`, configuration in
`deny.toml`. Anything not listed as an ignore fails the gate.

This file records *why* an advisory is tolerated, so a reviewer does not have to
reconstruct the reasoning from the config. Add a row before adding an ignore.

## Closed in the 2026-10-30 pass

| Advisory | Crate | Fix |
|---|---|---|
| RUSTSEC-2026-0187 | lopdf 0.31 | printpdf 0.7 to 0.12; both PDF renderers rewritten for the ops API |
| RUSTSEC-2024-0437 | protobuf 2.28 | prometheus 0.13 to 0.14 |
| RUSTSEC-2026-0391..0410 | zstd 0.13 / zstd-safe 7 | parquet 54 to 60, arrow 54 to 60 |
| RUSTSEC-2026-0194, -0195 | quick-xml 0.31 | quick-junit 0.3 to 0.8 |
| RUSTSEC-2026-0343..0349 | quinn-proto, rustls | updated to 0.11.19 and 0.23.46 |
| RUSTSEC-2026-0381, -0383 | webpki-roots, webpki-root-certs | 1.0.11 |
| (several) | zip 1.1, tokio, libz-sys | updated |

Audit count for the root lock: 30 advisories to 12.

## Open, with reasons

### cryptoki 0.7: RUSTSEC-2026-0286

Out-of-bounds read when decoding `CKA_ALLOWED_MECHANISMS` from a token. The fix
is cryptoki 0.10 or 0.11, but 0.11 makes `Session` deliberately `!Send` (it
holds a raw handle). `SthProducer` in `tlog-sequencer` carries the signer across
`tokio::spawn`, so the transparency sequencer stops compiling.

Reachable only through a compromised or hostile HSM. Removal requires
restructuring the signer so no PKCS#11 session crosses an await, for example by
running each signing operation in `spawn_blocking` with the session reacquired
inside. That is the next dependency item.

### zip 1.3 and 8.6: RUSTSEC-2025-0168, -2026-0367, -0368, -0369

All four concern archive **extraction**. This project never extracts an
untrusted archive:

- `docx-rs` writes `.docx` files; it does not read attacker input as a zip.
- `utoipa-swagger-ui` unpacks its own bundled Swagger distribution in a build
  script.

Pinned upstream: `docx-rs` 0.4.22 is the latest release, and it depends on zip 8.
`utoipa-swagger-ui` 7 is pinned by `utoipa` 4; moving to utoipa 5 is a macro
migration across roughly ten API crates and is tracked separately.

### rsa 0.9: RUSTSEC-2023-0071

The Marvin timing attack. There is no fixed 0.9 release. It is reached only
through `sqlx-mysql`. This project uses PostgreSQL exclusively and sets
`default-features = false` on `sqlx`, but `sqlx-mysql` still appears in
`Cargo.lock` through feature unification while `cargo tree` reports it
unreachable. `cargo deny` therefore does not see it and does not need an ignore;
`cargo audit` reads the lock and does.

Follow-up: determine what still activates the MySQL driver and make the lock
prune it. Until then, treat `cargo audit`'s count as an upper bound and
`cargo deny` as the graph-accurate gate.

### Unmaintained transitive crates

`atty`, `atomic-polyfill`, `bincode`, `derivative`, `number_prefix`, `paste`,
`proc-macro-error`, `rustls-pemfile`, `serde_cbor`, `ttf-parser`,
`tokio-tar`, `tracing-subscriber` 0.2, `unsafe-libyaml`, `async-std`.

All are transitive or build-time. Each ignore in `deny.toml` names the reason.
The two worth revisiting:

- `tracing-subscriber` 0.2 arrives through `ark-relations` (arkworks). If arkworks
  is upgraded, this disappears.
- `unsafe-libyaml` arrives through the deprecated `serde_yaml`. Migrating config
  parsing to `serde_yaml_ng` or `serde_norway` removes it.

## Reproducing

```bash
cargo deny check advisories     # graph-accurate gate; must print "advisories ok"
cargo audit --file Cargo.lock   # lockfile view; upper bound, includes stale entries
```
