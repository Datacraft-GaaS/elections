# Reports Templates

Template files for the EEL ABIS reports engine.

## Directory Layout

```
reports-templates/
  sql/          One .sql file per report slug (named after the sql_template_ref column value, minus the "templates/sql/" prefix)
  html/         Tera HTML templates
    base.html.tera              Full page skeleton (WCAG AA, CSP, no external assets)
    partials/
      header.html.tera          Page header block
      table.html.tera           Data table block
      chart.html.tera           Optional SVG chart block
      footer.html.tera          Footer block
  pdf/
    base-styles.md              Layout constants for the printpdf renderer
```

## SQL Template Contract

Each `.sql` file:
- Is a SELECT-only query; no DDL, no DML.
- Uses Tera-style named bind parameters: `{{ window_from }}`, `{{ window_to }}`, `{{ run_id }}`.
- Must return a deterministic column set (the extractor infers the schema from the first row).
- Should include inline comments describing each column.

Example usage in a template:
```sql
WHERE created_at >= '{{ window_from }}'::timestamptz
  AND created_at <  '{{ window_to }}'::timestamptz
```

## HTML Template Contract

Tera templates receive the following context variables:

| Variable           | Type               | Description                      |
|--------------------|--------------------|----------------------------------|
| `slug`             | `string`           | Report slug                      |
| `title`            | `string`           | Human-readable title             |
| `template_version` | `string`           | Semantic version                 |
| `window_from`      | `string` (RFC3339) | Start of reporting window        |
| `window_to`        | `string` (RFC3339) | End of reporting window          |
| `generated_at`     | `string` (RFC3339) | Generation timestamp             |
| `row_count`        | `usize`            | Number of data rows              |
| `columns`          | `Vec<String>`      | Ordered column names             |
| `rows`             | `Vec<Map>`         | Row objects keyed by column name |
| `chart_svg`        | `Option<String>`   | Inline SVG chart (if present)    |

## Adding a New Report

1. Add a row to `eel_report_definitions` (or via `V029__...sql` migration).
2. Create `reports-templates/sql/{slug}.sql` with the SELECT query.
3. If the report uses a custom HTML layout, create a Tera template and set `html_template_path`.
4. Bump `template_version` when the query or rendering changes semantically.
5. Run `cargo test -p eel-abis-reports` to verify all existing tests still pass.
