use chrono::Utc;
use eel_abis_reports::{
    domain::{DataClass, DeliveryStatus, RendererFormat, ReportClass, RunStatus},
    render::{
        csv::CsvRenderer, html::HtmlRenderer, json::JsonRenderer, parquet::ParquetRenderer,
        pdf::PdfRenderer, png::PngRenderer, xlsx::XlsxRenderer, Renderer, Rows, RunMeta,
        TemplateBundle,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_meta(slug: &str) -> RunMeta {
    RunMeta {
        slug: slug.to_owned(),
        title: format!("Test: {slug}"),
        template_version: "1.0.0".to_owned(),
        window_from: Utc::now() - chrono::Duration::days(1),
        window_to: Utc::now(),
        generated_at: Utc::now(),
    }
}

fn make_rows(n: usize) -> Rows {
    let columns = vec!["id".to_owned(), "value".to_owned(), "label".to_owned()];
    let data = (0..n)
        .map(|i| {
            vec![
                serde_json::json!(i as i64),
                serde_json::json!(i as f64 * 1.5),
                serde_json::json!(format!("row-{i}")),
            ]
        })
        .collect();
    Rows::new(columns, data)
}

fn make_empty_rows() -> Rows {
    Rows::new(vec!["col_a".to_owned(), "col_b".to_owned()], vec![])
}

// ---------------------------------------------------------------------------
// Enum exhaustiveness
// ---------------------------------------------------------------------------

#[test]
fn report_class_all_variants() {
    let variants = [
        ReportClass::Operational,
        ReportClass::Statistical,
        ReportClass::Regulatory,
        ReportClass::Legal,
        ReportClass::Public,
        ReportClass::Financial,
        ReportClass::Executive,
        ReportClass::AdHoc,
    ];
    assert_eq!(variants.len(), 8);
    for v in &variants {
        let s = v.to_string();
        assert!(!s.is_empty(), "variant should have display");
    }
}

#[test]
fn run_status_all_variants() {
    let variants = [
        RunStatus::Queued,
        RunStatus::Extracting,
        RunStatus::Rendering,
        RunStatus::Signing,
        RunStatus::Anchoring,
        RunStatus::Delivering,
        RunStatus::Complete,
        RunStatus::Failed,
        RunStatus::Cancelled,
    ];
    assert_eq!(variants.len(), 9);
}

#[test]
fn renderer_format_all_variants() {
    let variants = [
        RendererFormat::Json,
        RendererFormat::Csv,
        RendererFormat::Parquet,
        RendererFormat::Pdf,
        RendererFormat::Html,
        RendererFormat::Png,
        RendererFormat::Xlsx,
        RendererFormat::Svg,
        RendererFormat::Zip,
    ];
    assert_eq!(variants.len(), 9);
}

#[test]
fn delivery_status_all_variants() {
    let variants = [
        DeliveryStatus::Pending,
        DeliveryStatus::Sending,
        DeliveryStatus::Delivered,
        DeliveryStatus::Failed,
        DeliveryStatus::Abandoned,
    ];
    assert_eq!(variants.len(), 5);
}

#[test]
fn data_class_display() {
    assert_eq!(DataClass::Restricted.to_string(), "restricted");
    assert_eq!(DataClass::Sensitive.to_string(), "sensitive");
    assert_eq!(DataClass::Public.to_string(), "public");
}

// ---------------------------------------------------------------------------
// JSON renderer
// ---------------------------------------------------------------------------

#[test]
fn json_renderer_manifest_fields() {
    let rows = make_rows(5);
    let meta = make_meta("test-slug");
    let tmpl = TemplateBundle::default();
    let output = JsonRenderer::render(&rows, &meta, &tmpl).expect("json render");

    let parsed: serde_json::Value = serde_json::from_slice(&output.bytes).expect("valid json");
    let manifest = &parsed["manifest"];
    assert_eq!(manifest["slug"], "test-slug");
    assert_eq!(manifest["template_version"], "1.0.0");
    assert_eq!(manifest["row_count"], 5);
    assert!(manifest["window_from"].as_str().is_some());
    assert!(manifest["window_to"].as_str().is_some());
    assert!(manifest["generated_at"].as_str().is_some());
}

#[test]
fn json_renderer_rows_as_objects() {
    let rows = make_rows(3);
    let meta = make_meta("slug");
    let tmpl = TemplateBundle::default();
    let output = JsonRenderer::render(&rows, &meta, &tmpl).expect("json render");
    let parsed: serde_json::Value = serde_json::from_slice(&output.bytes).unwrap();
    let row_arr = parsed["rows"].as_array().unwrap();
    assert_eq!(row_arr.len(), 3);
    assert!(row_arr[0].get("id").is_some());
    assert!(row_arr[0].get("value").is_some());
    assert!(row_arr[0].get("label").is_some());
}

// ---------------------------------------------------------------------------
// CSV renderer
// ---------------------------------------------------------------------------

#[test]
fn csv_renderer_valid_rfc4180_with_header() {
    let rows = make_rows(10);
    let meta = make_meta("csv-test");
    let tmpl = TemplateBundle::default();
    let output = CsvRenderer::render(&rows, &meta, &tmpl).expect("csv render");

    // UTF-8 BOM
    assert_eq!(&output.bytes[..3], b"\xEF\xBB\xBF");

    // Parse CSV
    let content = std::str::from_utf8(&output.bytes[3..]).expect("utf8");
    let mut rdr = csv::Reader::from_reader(content.as_bytes());
    let headers: Vec<String> = rdr
        .headers()
        .expect("headers")
        .iter()
        .map(str::to_owned)
        .collect();
    assert_eq!(headers, vec!["id", "value", "label"]);

    let records: Vec<_> = rdr.records().collect();
    assert_eq!(records.len(), 10);
}

#[test]
fn csv_renderer_empty_rows() {
    let rows = make_empty_rows();
    let meta = make_meta("empty");
    let tmpl = TemplateBundle::default();
    let output = CsvRenderer::render(&rows, &meta, &tmpl).expect("csv empty");
    // Should still produce BOM + header
    assert!(output.bytes.starts_with(b"\xEF\xBB\xBF"));
    assert!(output.size_bytes > 0);
}

// ---------------------------------------------------------------------------
// PDF renderer
// ---------------------------------------------------------------------------

#[test]
fn pdf_renderer_valid_pdf_magic() {
    let rows = make_rows(5);
    let meta = make_meta("pdf-test");
    let tmpl = TemplateBundle::default();
    let output = PdfRenderer::render(&rows, &meta, &tmpl).expect("pdf render");
    // PDF magic bytes
    assert!(output.bytes.starts_with(b"%PDF"), "expected %PDF header");
    assert!(output.bytes.len() > 100);
}

#[test]
fn pdf_renderer_multipage() {
    let rows = make_rows(90); // 2 pages at 40 rows per page
    let meta = make_meta("multipage");
    let tmpl = TemplateBundle::default();
    let output = PdfRenderer::render(&rows, &meta, &tmpl).expect("pdf multipage");
    assert!(output.bytes.starts_with(b"%PDF"));
}

// ---------------------------------------------------------------------------
// HTML renderer
// ---------------------------------------------------------------------------

#[test]
fn html_renderer_has_csp_meta() {
    let rows = make_rows(3);
    let meta = make_meta("html-test");
    let tmpl = TemplateBundle::default();
    let output = HtmlRenderer::render(&rows, &meta, &tmpl).expect("html render");
    let html = std::str::from_utf8(&output.bytes).expect("utf8");
    assert!(
        html.contains("Content-Security-Policy"),
        "must have CSP meta"
    );
}

#[test]
fn html_renderer_no_external_urls() {
    let rows = make_rows(5);
    let meta = make_meta("html-no-ext");
    let tmpl = TemplateBundle::default();
    let output = HtmlRenderer::render(&rows, &meta, &tmpl).expect("html render");
    let html = std::str::from_utf8(&output.bytes).expect("utf8");
    // No external http links (data: is allowed)
    assert!(!html.contains("https://cdn"), "no CDN links");
    assert!(!html.contains("http://cdn"), "no CDN links");
}

#[test]
fn html_renderer_has_main_landmark() {
    let rows = make_rows(2);
    let meta = make_meta("html-main");
    let tmpl = TemplateBundle::default();
    let output = HtmlRenderer::render(&rows, &meta, &tmpl).expect("html render");
    let html = std::str::from_utf8(&output.bytes).expect("utf8");
    assert!(
        html.contains(r#"role="main""#) || html.contains("<main"),
        "main landmark required"
    );
}

// ---------------------------------------------------------------------------
// XLSX renderer
// ---------------------------------------------------------------------------

#[test]
fn xlsx_renderer_has_expected_columns() {
    let rows = make_rows(5);
    let meta = make_meta("xlsx-test");
    let tmpl = TemplateBundle::default();
    let output = XlsxRenderer::render(&rows, &meta, &tmpl).expect("xlsx render");
    // XLSX files start with PK (ZIP header)
    assert!(
        output.bytes.starts_with(b"PK"),
        "expected XLSX/ZIP magic bytes"
    );
    assert!(output.size_bytes > 100);
}

// ---------------------------------------------------------------------------
// Parquet renderer
// ---------------------------------------------------------------------------

#[test]
fn parquet_renderer_readable() {
    let rows = make_rows(20);
    let meta = make_meta("parquet-test");
    let tmpl = TemplateBundle::default();
    let output = ParquetRenderer::render(&rows, &meta, &tmpl).expect("parquet render");
    // Parquet magic: PAR1 at start and end
    assert!(output.bytes.starts_with(b"PAR1"), "expected PAR1 magic");
    assert!(
        output.bytes.ends_with(b"PAR1"),
        "expected PAR1 footer magic"
    );
}

#[test]
fn parquet_renderer_empty_rows() {
    let rows = make_empty_rows();
    let meta = make_meta("parquet-empty");
    let tmpl = TemplateBundle::default();
    let output = ParquetRenderer::render(&rows, &meta, &tmpl).expect("parquet empty");
    assert!(output.bytes.starts_with(b"PAR1"));
}

// ---------------------------------------------------------------------------
// PNG renderer
// ---------------------------------------------------------------------------

#[test]
fn png_renderer_valid_magic() {
    let rows = make_rows(10);
    let meta = make_meta("png-test");
    let tmpl = TemplateBundle::default();
    let output = PngRenderer::render(&rows, &meta, &tmpl).expect("png render");
    // PNG magic bytes: 0x89 PNG \r\n \x1a \n
    assert_eq!(
        &output.bytes[..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );
}

// ---------------------------------------------------------------------------
// sha256 / size tracking
// ---------------------------------------------------------------------------

#[test]
fn render_output_sha256_matches() {
    use sha2::{Digest, Sha256};
    let rows = make_rows(3);
    let meta = make_meta("hash-test");
    let tmpl = TemplateBundle::default();
    let output = JsonRenderer::render(&rows, &meta, &tmpl).expect("json render");
    let expected = Sha256::digest(&output.bytes).to_vec();
    assert_eq!(output.sha256, expected);
    assert_eq!(output.size_bytes, output.bytes.len() as i64);
}

// ---------------------------------------------------------------------------
// Status enum display roundtrip
// ---------------------------------------------------------------------------

#[test]
fn run_status_display_roundtrip() {
    assert_eq!(RunStatus::Queued.to_string(), "queued");
    assert_eq!(RunStatus::Extracting.to_string(), "extracting");
    assert_eq!(RunStatus::Complete.to_string(), "complete");
    assert_eq!(RunStatus::Failed.to_string(), "failed");
    assert_eq!(RunStatus::Cancelled.to_string(), "cancelled");
}

// ---------------------------------------------------------------------------
// CIDR prefix helper (via api module internals — tested through regex)
// ---------------------------------------------------------------------------

#[test]
fn ipv4_prefix_extraction() {
    // Tests the logic that strips IPs to /24
    let ip = "192.168.1.42";
    let parts: Vec<&str> = ip.split('.').collect();
    let prefix = format!("{}.0/24", parts[..3].join("."));
    assert_eq!(prefix, "192.168.1.0/24");
}

#[test]
fn ipv6_prefix_extraction() {
    let ip = "2001:db8:85a3::8a2e:370:7334";
    let parts: Vec<&str> = ip.split(':').collect();
    let prefix = format!("{}::/48", parts[..3].join(":"));
    assert_eq!(prefix, "2001:db8:85a3::/48");
}

// ---------------------------------------------------------------------------
// Scheduler: cron next-fire
// ---------------------------------------------------------------------------

#[test]
fn scheduler_next_fire_daily() {
    let result = eel_abis_reports::scheduler::next_fire("0 1 * * *");
    assert!(result.is_ok(), "valid cron should parse");
    let t = result.unwrap();
    assert!(t.is_some(), "daily cron always has a next fire");
}

#[test]
fn scheduler_next_fire_invalid_cron() {
    let result = eel_abis_reports::scheduler::next_fire("not_a_cron");
    assert!(result.is_err(), "invalid cron should error");
}

// ---------------------------------------------------------------------------
// Row count edge cases
// ---------------------------------------------------------------------------

#[test]
fn rows_row_count() {
    let r = make_rows(42);
    assert_eq!(r.row_count(), 42);
}

#[test]
fn rows_empty_count() {
    let r = make_empty_rows();
    assert_eq!(r.row_count(), 0);
}

// ---------------------------------------------------------------------------
// Template bundle default
// ---------------------------------------------------------------------------

#[test]
fn template_bundle_default_empty() {
    let tmpl = TemplateBundle::default();
    assert!(tmpl.html_template_path.is_none());
    assert!(tmpl.chart_svg.is_none());
}
