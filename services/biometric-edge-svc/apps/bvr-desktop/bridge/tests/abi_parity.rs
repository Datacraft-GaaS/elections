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

//! The C header must declare every function the bridge exports.
//!
//! This exists because it already went wrong once: `eel_string_free` was
//! implemented and exported, and the C++ side calls it from its string deleter
//! on every returned string, but it was never declared in `eel_bvr.h`. The Qt
//! client therefore failed to compile, and had it compiled against an implicit
//! declaration the result would have been undefined behaviour on every string
//! the library returns.
//!
//! Symbols are read from the bridge source rather than from a built archive, so
//! the test needs no prior `cargo build`, no `nm`, and no platform-specific
//! tooling. An earlier version read the static library and had to fail when the
//! library was absent, which made it fail on a clean checkout -- a test that
//! needs a manual build step before it can run is a test people disable.

use std::collections::BTreeSet;

/// The ABI surface, pinned explicitly.
///
/// Removing a function from the ABI must be a deliberate edit here rather than a
/// silent consequence of deleting a `#[no_mangle]`.
const EXPECTED: &[&str] = &[
    "eel_store_append",
    "eel_store_check_key",
    "eel_store_free",
    "eel_store_instance_id",
    "eel_store_mark_synced",
    "eel_store_new",
    "eel_store_pending_batch",
    "eel_store_public_key",
    "eel_store_seal_to_disk",
    "eel_store_sync_state",
    "eel_store_value",
    "eel_store_verify",
    "eel_string_free",
    "eel_workflow_capture_json",
    "eel_workflow_field_json",
    "eel_workflow_free",
    "eel_workflow_id",
    "eel_workflow_load",
    "eel_workflow_step_count",
    "eel_workflow_step_field_count",
    "eel_workflow_step_kind",
    "eel_workflow_step_label",
    "eel_workflow_validate",
];

fn bridge_source() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {} failed: {e}", path.display()))
}

fn header_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("include/eel_bvr.h");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {} failed: {e}", path.display()))
}

/// Removes `/* ... */` and `// ...` comments so prose is never counted.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_block = false;
    let mut in_line = false;
    let mut in_string = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if in_block {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block = false;
            }
            continue;
        }
        if in_line {
            if c == '\n' {
                in_line = false;
                out.push(c);
            }
            continue;
        }
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                in_block = true;
            }
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                in_line = true;
            }
            '"' => {
                in_string = true;
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// Every `eel_*` identifier followed by `(`.
///
/// Matching on a following parenthesis works for both single-line and wrapped
/// prototypes, and after comment stripping nothing but a declaration remains.
fn declared_functions() -> BTreeSet<String> {
    let stripped = strip_comments(&header_text());
    let chars: Vec<char> = stripped.chars().collect();
    let mut found = BTreeSet::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i..].starts_with(&['e', 'e', 'l', '_']) {
            let start = i;
            let mut end = i + 4;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            let name: String = chars[start..end].iter().collect();
            let mut probe = end;
            while probe < chars.len() && chars[probe].is_whitespace() {
                probe += 1;
            }
            if probe < chars.len() && chars[probe] == '(' {
                found.insert(name);
            }
            i = end;
        } else {
            i += 1;
        }
    }
    found
}

/// Every `#[no_mangle]` function the bridge source exports.
fn exported_functions() -> BTreeSet<String> {
    let stripped = strip_comments(&bridge_source());
    let mut found = BTreeSet::new();
    let mut lines = stripped.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() != "#[no_mangle]" {
            continue;
        }
        // The attribute may be followed by another attribute, then the fn.
        let mut name = None;
        for candidate in lines.by_ref() {
            if let Some(position) = candidate.find("fn ") {
                let rest = &candidate[position + 3..];
                let end = rest
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len());
                name = Some(rest[..end].to_owned());
                break;
            }
        }
        if let Some(name) = name {
            if name.starts_with("eel_") {
                found.insert(name);
            }
        }
    }
    found
}

#[test]
fn the_bridge_source_actually_exports_the_expected_surface() {
    // Guards the parser itself: if `exported_functions` stopped finding
    // anything, the two comparison tests below would pass vacuously.
    let exported = exported_functions();
    assert!(
        exported.len() >= EXPECTED.len(),
        "parser found only {} symbols, expected at least {}: {exported:?}",
        exported.len(),
        EXPECTED.len()
    );
    for name in EXPECTED {
        assert!(
            exported.contains(*name),
            "{name} is pinned in EXPECTED but not exported by src/lib.rs"
        );
    }
}

#[test]
fn the_header_declares_every_exported_symbol() {
    let exported = exported_functions();
    let declared = declared_functions();
    let undeclared: Vec<_> = exported.difference(&declared).collect();
    assert!(
        undeclared.is_empty(),
        "exported but not declared in include/eel_bvr.h: {undeclared:?}. \
         A C consumer cannot call these."
    );
}

#[test]
fn nothing_is_declared_that_is_not_exported() {
    let exported = exported_functions();
    let declared = declared_functions();
    let phantom: Vec<_> = declared.difference(&exported).collect();
    assert!(
        phantom.is_empty(),
        "declared in the header but not exported: {phantom:?}. \
         A C consumer would link-fail on these."
    );
}

#[test]
fn the_string_free_declaration_is_present_and_documented() {
    // The one that bit us. Asserted separately so the reason it exists sits
    // next to the assertion rather than only in a comment.
    let text = header_text();
    assert!(
        text.contains("void eel_string_free(char *s);"),
        "the free function must be declared verbatim"
    );
    assert!(
        text.contains("owned by Rust"),
        "the declaration should say returned strings are Rust-owned"
    );
}
