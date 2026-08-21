# eel-synth

Deterministic synthetic data generator for the eel IEMS. Produces voters, polling stations,
constituencies, counties, Form 34A images, KIEMS kits, personnel, and diaspora embassy records.

## Quick start

```sh
cargo build --release -p eel-synth

# Default profile: 100k voters, 4600 stations, 200 forms, 6500 kits, 25000 personnel.
./target/release/eel-synth all --seed 42 --out synth_out/

# Specific generators.
./target/release/eel-synth voters --seed 42 --scale 1000 --format csv
./target/release/eel-synth geo    --seed 42 --stations 4600 --format geojson
./target/release/eel-synth forms  --seed 42 --scale 50 --out forms_out/
./target/release/eel-synth kits   --seed 42 --scale 500
./target/release/eel-synth personnel --seed 42 --scale 1000 --format sql
./target/release/eel-synth embassies --seed 42

# Determinism check.
./target/release/eel-synth verify --seed 42
```

## Subcommands

| Subcommand | Description |
|---|---|
| `voters` | Synthetic voter register. CSV, SQL COPY. |
| `geo` | Polling stations (46229 max), 290 constituencies, 47 counties. GeoJSON or SQL COPY. |
| `forms` | Form 34A PNG images with optional degradation. |
| `kits` | KIEMS device records with TPM key stubs and geo-fence assignments. |
| `personnel` | Election officials with role assignments and station bindings. |
| `embassies` | Diaspora voting locations at Kenyan diplomatic missions. |
| `all` | Run all generators with a named profile. |
| `verify` | Compute SHA-256 hash of default output and compare to stored checkpoint. |

## Scale profiles

| Profile | Voters | Stations | Forms | Kits | Personnel | Embassies |
|---|---|---|---|---|---|---|
| `default` | 100,000 | 4,600 | 200 | 6,500 | 25,000 | 22 |
| `smoke` | 100 | 10 | 5 | 20 | 50 | 5 |

Pass `--scale default` or `--scale smoke` to the `all` and `verify` subcommands.

## Seed policy

- `--seed` accepts any u64. The default is `42`.
- The same seed on the same build produces byte-identical output.
- Each generator derives an independent sub-seed via deterministic mixing, so changing
  voter count does not perturb kit output.
- The canonical checkpoint seed is `42` with the `default` profile.

On first run, `verify --seed 42` writes `fixtures/checksums/seed42-default.sha256` and
exits 0. Subsequent runs compare against the stored hash.

## Output formats

- `csv`: RFC 4180 CSV with header row. Written by `csv::Writer`.
- `sql`: PostgreSQL `COPY table FROM STDIN` blocks, pipe to `psql`.
- `parquet`: not supported in this build. Use `--format csv`.

## Form 34A images

PNG files rendered with a fixed-layout bitmap font. All pixel operations are deterministic.

Degradation flags (comma-separated, pass to `--degraded`):

- `torn`: simulates a paper tear along the right edge.
- `wet`: adds dark blotches simulating water damage.
- `angled`: rotates the form by a small random angle.
- `low-light`: darkens the image to simulate poor scan conditions.

Example:

```sh
eel-synth forms --seed 42 --scale 10 --out forms_out/ --degraded torn,wet,angled
```

### Cross-arch determinism

PNG output uses the `image` crate's pure-Rust `miniz_oxide` deflate backend. Byte identity
holds within a single architecture. Cross-arch PNG byte identity is not guaranteed due to
potential differences in DEFLATE block sizing. Hash checkpoints in CI are pinned per target
triple. Wave 5 reproducibility work resolves cross-arch form-image determinism.

## PII guarantee

No real personal data appears anywhere in this crate.

- National IDs start at `999_000_000` (reserved synthetic range, ITU-T / Kenya DRB convention).
- Phones use `+254799xxxxxx` (reserved test range, not assigned to any subscriber).
- All names are generated from `fixtures/first_names.txt` and `fixtures/last_names.txt`,
  which contain only invented names. No real identities are included.
- Geo coordinates are procedurally generated within county bounding boxes from public data.
- Embassy locations are public-domain (Class C) diplomatic mission addresses.

To confirm: `grep -r "class-A\|PIIRestricted" tools/synth/` returns no hits.

## Extending the fixture

To add more names, append lines to `fixtures/first_names.txt` or `fixtures/last_names.txt`.
Guidelines:

1. Use plausible Kenyan-pattern names only.
2. Avoid names of known politicians, celebrities, or public figures.
3. After changes, run `eel-synth verify --seed 42` to update the checkpoint.

To add a new generator, implement a module under `src/`, export from `lib.rs`, add a
subcommand in `main.rs`, and add property tests in `tests/determinism.rs`.

## Running tests

```sh
cargo test -p eel-synth
cargo test -p eel-synth --test determinism
cargo test -p eel-synth --test smoke
```
