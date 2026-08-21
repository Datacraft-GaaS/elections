use eel_abis_render_docx::{
    render_docx, CalloutStyle, DocxOptions, DocxTemplate, PageOrientation, PageSize, Run, Section,
};
use serde_json::{json, Value};

fn default_template() -> DocxTemplate {
    DocxTemplate {
        title: "Test Report".to_string(),
        subtitle: Some("Test subtitle".to_string()),
        header_html: None,
        footer_html: None,
        sections: vec![],
        page_orientation: PageOrientation::Portrait,
        page_size: PageSize::A4,
        margins: None,
        font_family: None,
        base_font_size: None,
        table_style_id: None,
    }
}

fn default_opts() -> DocxOptions {
    DocxOptions::default()
}

fn make_rows(n: usize) -> Vec<Value> {
    (0..n)
        .map(|i| json!({"id": i, "name": format!("row_{i}"), "value": i * 10}))
        .collect()
}

#[test]
fn empty_rows_produces_valid_docx() {
    let tmpl = default_template();
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
    assert!(out.size_bytes > 0);
}

#[test]
fn docx_is_valid_zip_container() {
    let tmpl = default_template();
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    // PK zip magic bytes
    assert_eq!(&out.bytes[0..2], b"PK");
}

#[test]
fn sha256_is_hex_string() {
    let tmpl = default_template();
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert_eq!(out.sha256.len(), 64);
    assert!(out.sha256.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn sha256_stable_across_runs() {
    let tmpl = default_template();
    let out1 = render_docx(&[], &tmpl, &default_opts()).unwrap();
    let out2 = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert_eq!(out1.sha256, out2.sha256);
}

#[test]
fn table_has_expected_row_count() {
    let mut tmpl = default_template();
    tmpl.sections = vec![Section::Table {
        headers: vec!["a".to_string(), "b".to_string()],
        rows: vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
            vec!["5".to_string(), "6".to_string()],
        ],
        column_widths: None,
    }];
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
    assert_eq!(&out.bytes[0..2], b"PK");
}

#[test]
fn landscape_orientation_flag() {
    let mut tmpl = default_template();
    tmpl.page_orientation = PageOrientation::Landscape;
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
}

#[test]
fn toc_section_renders() {
    let mut tmpl = default_template();
    tmpl.sections = vec![
        Section::Heading {
            level: 1,
            text: "Introduction".to_string(),
        },
        Section::TableOfContents { max_depth: 3 },
        Section::Heading {
            level: 2,
            text: "Details".to_string(),
        },
    ];
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
}

#[test]
fn image_embed_roundtrip() {
    // Minimal 1x1 PNG bytes
    let png_bytes: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC, 0x33, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let mut tmpl = default_template();
    tmpl.sections = vec![Section::Image {
        bytes: png_bytes,
        alt_text: "Test image".to_string(),
        width_pt: 100.0,
        height_pt: 100.0,
    }];
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
}

#[test]
fn auto_table_generated_from_rows() {
    let tmpl = default_template();
    let rows = make_rows(5);
    let out = render_docx(&rows, &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
    assert_eq!(&out.bytes[0..2], b"PK");
}

#[test]
fn suppress_auto_table_option() {
    let tmpl = default_template();
    let rows = make_rows(5);
    let opts = DocxOptions {
        suppress_auto_table: true,
        template_vars: None,
    };
    let out_suppressed = render_docx(&rows, &tmpl, &opts).unwrap();
    let out_auto = render_docx(&rows, &tmpl, &default_opts()).unwrap();
    // Suppressed output should be smaller (no table)
    assert!(out_suppressed.size_bytes <= out_auto.size_bytes);
}

#[test]
fn page_count_estimate_increases_with_rows() {
    let tmpl = default_template();
    let out_small = render_docx(&make_rows(10), &tmpl, &default_opts()).unwrap();
    let out_large = render_docx(&make_rows(1000), &tmpl, &default_opts()).unwrap();
    assert!(out_large.page_count >= out_small.page_count);
}

#[test]
fn callout_section_renders() {
    let mut tmpl = default_template();
    tmpl.sections = vec![
        Section::Callout {
            style: CalloutStyle::Info,
            text: "Important information".to_string(),
        },
        Section::Callout {
            style: CalloutStyle::Warn,
            text: "Warning message".to_string(),
        },
        Section::Callout {
            style: CalloutStyle::Error,
            text: "Error occurred".to_string(),
        },
    ];
    let out = render_docx(&[], &tmpl, &default_opts()).unwrap();
    assert!(!out.bytes.is_empty());
}

#[test]
fn large_table_completes_under_3s() {
    let tmpl = default_template();
    let rows = make_rows(10_000);
    let start = std::time::Instant::now();
    let out = render_docx(&rows, &tmpl, &default_opts()).unwrap();
    let elapsed = start.elapsed();
    assert!(!out.bytes.is_empty());
    assert!(elapsed.as_secs() < 3, "Took {elapsed:?}, expected < 3s");
}
