# abis-render-docx

Standalone DOCX renderer for the ABIS reports engine. Transforms adjudication decision data into formatted Microsoft Word documents for printing and archival.

## What it does

- Template rendering: Loads Jinja2 DOCX templates from `reports-templates/` directory; substitutes case data, match scores, and decision rationale
- Structured output: Produces .docx files with preserved formatting, page breaks, images (embedded candidate photos), and tables (match candidate lists)
- Signature insertion: Embeds HSM-signed hash into document footer as proof of non-repudiation
- Batch rendering: Renders multiple adjudication decisions in parallel; streams output to MinIO
- Error recovery: Validates template syntax at startup; falls back to text-only rendering on image embed failure

## Public API surface

```rust
pub struct DocumentContext {
    pub case_id: String,
    pub match_score: f32,
    pub decision: CaseDecision,
    pub adjudicator_name: String,
    pub adjudication_date: DateTime,
    pub candidate_photos: Vec<Vec<u8>>,
}

pub struct RenderedDocument {
    pub docx_bytes: Vec<u8>,
    pub file_size: usize,
}

pub async fn render_decision(ctx: DocumentContext) -> Result<RenderedDocument>;
pub async fn render_batch(cases: &[DocumentContext]) -> Result<Vec<RenderedDocument>>;
pub fn validate_templates() -> Result<()>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (Jinja2 rendering, image embedding, signature insertion)
cargo test --lib --package eel-abis-render-docx

# Integration tests (template file I/O, batch rendering, MinIO upload)
cargo test --test '*' --package eel-abis-render-docx -- --test-threads=1
```

## Bench commands

```bash
# Single document rendering (case data, 5 candidate photos)
cargo bench --package eel-abis-render-docx -- render_single

# Batch rendering (100 documents, parallel)
cargo bench --package eel-abis-render-docx -- render_batch
```

## Dependencies

- `docx-rs` - DOCX file structure and XML serialization
- `askama` (or `tera`) - Jinja2-compatible template rendering
- `image` - Decode and embed candidate photos (JPEG/PNG)
- `tokio` - Async rendering pool
- `sha2` - Hash document for integrity check

## Related docs

- [reports-templates/](../../reports-templates) - DOCX template files and variable reference
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Template update procedures, image embedding troubleshooting
