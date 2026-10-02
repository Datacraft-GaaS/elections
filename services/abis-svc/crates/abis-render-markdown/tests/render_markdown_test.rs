// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use eel_abis_render_markdown::{
    render_markdown, MarkdownOptions, MarkdownTemplate, MdSection, TableAlignment,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn default_template() -> MarkdownTemplate {
    MarkdownTemplate {
        title: "Test Report".to_string(),
        front_matter: BTreeMap::new(),
        sections: vec![],
        gfm: true,
        include_toc: false,
        table_alignment: TableAlignment::Left,
    }
}

fn default_opts() -> MarkdownOptions {
    MarkdownOptions::default()
}

fn make_rows(n: usize) -> Vec<Value> {
    (0..n)
        .map(|i| json!({"id": i, "name": format!("row_{i}"), "value": i * 10}))
        .collect()
}

#[test]
fn empty_rows_produces_valid_md_with_title() {
    let tmpl = default_template();
    let out = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    let text = String::from_utf8(out.bytes).unwrap();
    assert!(text.contains("# Test Report"));
    assert!(out.size_bytes > 0);
}

#[test]
fn sha256_stable_across_runs() {
    let tmpl = default_template();
    let out1 = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    let out2 = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    assert_eq!(out1.sha256, out2.sha256);
}

#[test]
fn sha256_is_hex_string() {
    let tmpl = default_template();
    let out = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    assert_eq!(out.sha256.len(), 64);
    assert!(out.sha256.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn auto_toc_anchors_match_heading_slugs() {
    let mut tmpl = default_template();
    tmpl.include_toc = true;
    tmpl.sections = vec![
        MdSection::Heading {
            level: 1,
            text: "Introduction".to_string(),
        },
        MdSection::Heading {
            level: 2,
            text: "Data Details".to_string(),
        },
    ];
    let out = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    let text = String::from_utf8(out.bytes).unwrap();
    assert!(text.contains("#introduction"));
    assert!(text.contains("#data-details"));
}

#[test]
fn gfm_table_alignment_respected() {
    let mut tmpl = default_template();
    tmpl.gfm = true;
    tmpl.sections = vec![MdSection::Table {
        headers: vec!["Name".to_string(), "Value".to_string()],
        rows: vec![vec!["Alice".to_string(), "42".to_string()]],
        alignment: Some(vec![TableAlignment::Left, TableAlignment::Right]),
    }];
    let out = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    let text = String::from_utf8(out.bytes).unwrap();
    assert!(text.contains(":---")); // left
    assert!(text.contains("---:")); // right
}

#[test]
fn commonmark_strict_no_gfm_tables() {
    let mut tmpl = default_template();
    tmpl.gfm = false;
    tmpl.sections = vec![MdSection::Table {
        headers: vec!["A".to_string()],
        rows: vec![vec!["1".to_string()]],
        alignment: None,
    }];
    let out = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    let text = String::from_utf8(out.bytes).unwrap();
    // In strict mode the table is rendered as a code block, not a GFM table
    assert!(text.contains("```"));
    assert!(!text.contains("| A |"));
}

#[test]
fn front_matter_emitted_only_when_opted() {
    let mut tmpl = default_template();
    tmpl.front_matter
        .insert("author".to_string(), json!("Alice"));

    let opts_no_fm = MarkdownOptions {
        emit_front_matter: false,
        ..Default::default()
    };
    let out_no = render_markdown(&[], &tmpl, &opts_no_fm).unwrap();
    let text_no = String::from_utf8(out_no.bytes).unwrap();
    assert!(!text_no.contains("---"));

    let opts_fm = MarkdownOptions {
        emit_front_matter: true,
        ..Default::default()
    };
    let out_yes = render_markdown(&[], &tmpl, &opts_fm).unwrap();
    let text_yes = String::from_utf8(out_yes.bytes).unwrap();
    assert!(text_yes.starts_with("---\n"));
    assert!(text_yes.contains("author:"));
}

#[test]
fn utf8_bom_option() {
    let tmpl = default_template();
    let opts_bom = MarkdownOptions {
        utf8_bom: true,
        ..Default::default()
    };
    let out = render_markdown(&[], &tmpl, &opts_bom).unwrap();
    // UTF-8 BOM is EF BB BF
    assert_eq!(&out.bytes[0..3], b"\xef\xbb\xbf");
}

#[test]
fn auto_table_from_rows() {
    let tmpl = default_template();
    let rows = make_rows(5);
    let out = render_markdown(&rows, &tmpl, &default_opts()).unwrap();
    let text = String::from_utf8(out.bytes).unwrap();
    assert!(text.contains("| id |") || text.contains("| name |"));
}

#[test]
fn large_10k_row_table_completes() {
    let tmpl = default_template();
    let rows = make_rows(10_000);
    let start = std::time::Instant::now();
    let out = render_markdown(&rows, &tmpl, &default_opts()).unwrap();
    let elapsed = start.elapsed();
    assert!(!out.bytes.is_empty());
    assert!(elapsed.as_secs() < 3, "Took {elapsed:?}, expected < 3s");
}

#[test]
fn round_trip_parse_validates() {
    let mut tmpl = default_template();
    tmpl.sections = vec![
        MdSection::Heading {
            level: 1,
            text: "Overview".to_string(),
        },
        MdSection::Paragraph {
            text: "Some description text.".to_string(),
        },
        MdSection::Table {
            headers: vec!["Col1".to_string(), "Col2".to_string()],
            rows: vec![vec!["a".to_string(), "b".to_string()]],
            alignment: None,
        },
    ];
    let out = render_markdown(&[], &tmpl, &default_opts()).unwrap();
    // If we got here without error, round-trip validation passed
    assert!(!out.bytes.is_empty());
}
