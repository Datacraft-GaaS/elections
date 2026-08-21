# eel Rust API Client

Generated Rust clients for all eel OpenAPI contexts.

## Regenerate

From the repo root:

```sh
just gen-clients
```

Or from this directory:

```sh
./generate.sh --regen
```

## What the generated code contains

For each OpenAPI context (e.g. `elections`, `results`), a Rust crate is generated under `generated/<ctx>/` using either `progenitor-cli` (preferred) or `openapi-generator-cli` with the `rust` template. Each crate includes:

- Strongly typed structs for all schemas
- An async client using `reqwest`
- Builder-pattern request types

Crate name: `eel-api-client-<ctx>`

## Usage

Add the generated crate to your `Cargo.toml`:

```toml
[dependencies]
eel-api-client-elections = { path = "../api-clients/rs/generated/elections" }
```

Then use it:

```rust
use eel_api_client_elections::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("https://api.eel.example.com");
    let elections = client.list_elections().await.unwrap();
}
```

## Notes

- The `generated/` directory is gitignored. Run `./generate.sh --regen` after pulling new OpenAPI specs.
- Prefers `progenitor-cli` if available (`cargo install progenitor-cli`). Falls back to `openapi-generator-cli` via npx.
- `rustfmt` is applied to all generated `.rs` files if available.
