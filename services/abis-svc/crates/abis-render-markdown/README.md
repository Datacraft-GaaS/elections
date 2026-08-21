# abis-render-markdown

Standalone Markdown renderer for the ABIS reports engine. Transforms adjudication decision data into human-readable Markdown documents for email delivery and web portal display.

## What it does

- Template rendering: Loads Markdown templates from `reports-templates/` directory; substitutes case data, scores, and decision text
- Structured output: Produces .md files with headers, tables (candidate lists), code blocks (JSON match metadata), and embedded data URIs for images
- Email format: Generates MIME multipart emails with inline HTML rendering and plain text fallback
- HTML conversion: Optionally transpiles Markdown to HTML for web portal display (via `pulldown-cmark`)
- Batch rendering: Renders multiple decisions in parallel; streams to email queue or web storage

## Public API surface

```rust
pub struct DocumentContext {
    pub case_id: String,
    pub match_score: f32,
    pub decision: CaseDecision,
    pub adjudicator_name: String,
    pub adjudication_date: DateTime,
    pub candidate_list: Vec<Candidate>,
}

pub struct RenderedMarkdown {
    pub markdown: String,
    pub html: Option<String>,
}

pub async fn render_decision(ctx: DocumentContext) -> Result<RenderedMarkdown>;
pub async fn render_batch(cases: &[DocumentContext]) -> Result<Vec<RenderedMarkdown>>;
pub fn render_to_html(markdown: &str) -> Result<String>;
```

## Feature flags

None.

## Test commands

```bash
# Unit tests (template substitution, Markdown syntax validation)
cargo test --lib --package eel-abis-render-markdown

# Integration tests (file I/O, HTML transpilation, email MIME generation)
cargo test --test '*' --package eel-abis-render-markdown -- --test-threads=1
```

## Bench commands

```bash
# Single Markdown render (case data, candidate list)
cargo bench --package eel-abis-render-markdown -- render_single

# HTML transpilation (100 Markdown documents)
cargo bench --package eel-abis-render-markdown -- markdown_to_html
```

## Dependencies

- `pulldown-cmark` - Markdown to HTML transpilation
- `handlebars` (or `askama`) - Template rendering with conditional blocks
- `serde` - Context serialization
- `tokio` - Async batch rendering pool

## Related docs

- [reports-templates/](../../reports-templates) - Markdown template files and variable reference
- [OPERATIONS.md](../../docs/OPERATIONS.md) - Template update procedures, HTML rendering troubleshooting
