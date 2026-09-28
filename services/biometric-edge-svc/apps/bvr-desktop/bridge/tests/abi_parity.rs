//! The C header must declare every symbol the bridge exports.
//!
//! This exists because it already went wrong once: `eel_string_free` was
//! implemented and exported, and the C++ side called it from its string deleter
//! on every returned string, but it was never declared in `eel_bvr.h`. The Qt
//! client therefore failed to compile, and had it compiled against an implicit
//! declaration in C89 the result would have been undefined behaviour on every
//! string the library returned.
//!
//! Checking symbol names only, not signatures. Signature drift is caught by the
//! C++ build, which is the stricter check, but only if someone builds the Qt
//! client. This test runs in the ordinary workspace test suite.

use std::collections::BTreeSet;
use std::process::Command;

/// Symbols the bridge is expected to export. Kept explicit so that removing a
/// function from the ABI is a deliberate edit here rather than a silent
/// consequence of deleting a `#[no_mangle]`.
const EXPECTED: &[&str] = &[
    "eel_store_append",
    "eel_store_free",
    "eel_store_instance_id",
    "eel_store_mark_synced",
    "eel_store_new",
    "eel_store_pending_batch",
    "eel_store_public_key",
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

/// Locates the built static library, or returns None when the bridge has not
/// been built yet (a fresh checkout running only `cargo test`).
fn static_library() -> Option<std::path::PathBuf> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    // bridge/ -> bvr-desktop/ -> apps/ -> biometric-edge-svc/ -> services/ -> repo root
    let root = manifest.join("../../../../..");
    for candidate in [
        root.join("target/debug/libeel_bvr_bridge.a"),
        root.join("target/release/libeel_bvr_bridge.a"),
        manifest.join("target/debug/libeel_bvr_bridge.a"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Every `eel_*` function the static library defines.
fn exported_symbols() -> BTreeSet<String> {
    let Some(library) = static_library() else {
        return BTreeSet::new();
    };
    let output = Command::new("nm")
        .args(["-gU", library.to_str().expect("library path is UTF-8")])
        .output()
        .expect("nm must be available to check the ABI");
    assert!(output.status.success(), "nm failed on {}", library.display());

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            // nm on an archive prefixes each member with a dashed rule, so the
            // line looks like "---------------- T _eel_workflow_load". Find the
            // type code rather than assuming a fixed column.
            let fields: Vec<&str> = line.split_whitespace().collect();
            let position = fields.iter().position(|f| *f == "T")?;
            let name = fields.get(position + 1)?;
            // macOS prefixes C symbols with an underscore.
            Some(name.trim_start_matches('_').to_owned())
        })
        .filter(|name| name.starts_with("eel_"))
        .collect()
}

/// Every `eel_*` identifier the header declares.
///
/// Scans for an identifier immediately followed by `(`, which matches a
/// prototype whether it is on one line or wrapped across several. Header files
/// contain declarations only, so no call site can be mistaken for one.
fn declared_functions() -> BTreeSet<String> {
    let header = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("include/eel_bvr.h");
    let text = std::fs::read_to_string(&header)
        .unwrap_or_else(|e| panic!("reading {} failed: {e}", header.display()));

    // Strip block comments so prose mentioning a function is not counted.
    let mut stripped = String::with_capacity(text.len());
    let mut in_comment = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if in_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_comment = false;
            }
            continue;
        }
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_comment = true;
            continue;
        }
        stripped.push(c);
    }

    let bytes: Vec<char> = stripped.chars().collect();
    let mut found = BTreeSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 'e' && bytes[i..].starts_with(&['e', 'e', 'l', '_']) {
            let start = i;
            let mut end = i + 4;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == '_') {
                end += 1;
            }
            let name: String = bytes[start..end].iter().collect();
            // Look past whitespace for the opening parenthesis of a prototype.
            let mut probe = end;
            while probe < bytes.len() && bytes[probe].is_whitespace() {
                probe += 1;
            }
            if probe < bytes.len() && bytes[probe] == '(' {
                found.insert(name);
            }
            i = end;
        } else {
            i += 1;
        }
    }
    found
}

#[test]
fn the_header_declares_every_exported_symbol() {
    let exported = exported_symbols();
    assert!(
        !exported.is_empty(),
        "could not read the exported symbols; the static library was not found. \
         Run `cargo build -p eel-bvr-bridge` before this test, otherwise it would \
         pass without checking anything."
    );
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
    let exported = exported_symbols();
    assert!(!exported.is_empty(), "no symbols read; would pass vacuously");
    let declared = declared_functions();
    let phantom: Vec<_> = declared.difference(&exported).collect();
    assert!(
        phantom.is_empty(),
        "declared in the header but not exported: {phantom:?}. \
         A C consumer would link-fail on these."
    );
}

#[test]
fn the_expected_abi_surface_is_present() {
    // Guards against someone deleting a #[no_mangle] and the test above
    // happily agreeing with the smaller set.
    let declared = declared_functions();
    for name in EXPECTED {
        assert!(
            declared.contains(*name),
            "{name} must be declared in include/eel_bvr.h"
        );
    }
}

#[test]
fn the_string_free_declaration_is_present_and_documented() {
    // The one that bit us. Asserted separately so the reason it exists is
    // recorded next to the assertion rather than only in a comment above.
    let header = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("include/eel_bvr.h");
    let text = std::fs::read_to_string(&header).unwrap();
    assert!(
        text.contains("void eel_string_free(char *s);"),
        "the free function must be declared verbatim"
    );
    assert!(
        text.contains("eel_string_free") && text.contains("owned by Rust"),
        "the declaration should say that returned strings are Rust-owned"
    );
}
