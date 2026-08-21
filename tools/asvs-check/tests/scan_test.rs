use eel_asvs_check_lib::{manifest, scan};
use std::fs;
use tempfile::TempDir;

fn minimal_manifest(dir: &TempDir) -> std::path::PathBuf {
    let content = r#"requirements:
  - id: V6.2.3
    level: L3
    description: "Test requirement V6.2.3"
    mapped_tags: []
  - id: V2.1.1
    level: L3
    description: "Test requirement V2.1.1"
    mapped_tags: []
  - id: V1.2.1
    level: L2
    description: "Test requirement V1.2.1"
    mapped_tags: []
"#;
    let path = dir.path().join("asvs-manifest.yaml");
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_rust_trace_tag_covers_requirement() {
    let dir = TempDir::new().unwrap();
    let manifest_path = minimal_manifest(&dir);

    // Write a fake Rust test file with ASVS trace tags
    let test_src = r#"
#[cfg(test)]
mod tests {
    #[trace("ASVS-V6.2.3", "ASVS-V2.1.1")]
    #[test]
    fn test_crypto_init_vector() {
        assert!(true);
    }
}
"#;
    let test_dir = dir.path().join("services/some-svc/src");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join("tests.rs"), test_src).unwrap();

    let requirements = manifest::load(&manifest_path).unwrap();
    let tag_hits = scan::scan_repo(dir.path()).unwrap();
    let results = scan::map_requirements(&requirements, &tag_hits);

    let v623 = results
        .iter()
        .find(|r| r.requirement.id == "V6.2.3")
        .unwrap();
    let v211 = results
        .iter()
        .find(|r| r.requirement.id == "V2.1.1")
        .unwrap();
    let v121 = results
        .iter()
        .find(|r| r.requirement.id == "V1.2.1")
        .unwrap();

    assert!(v623.covered, "V6.2.3 should be covered by ASVS-V6.2.3 tag");
    assert!(v211.covered, "V2.1.1 should be covered by ASVS-V2.1.1 tag");
    assert!(!v121.covered, "V1.2.1 has no tag so should be uncovered");

    assert!(!v623.hits.is_empty());
    assert!(v623.hits[0].location.contains("tests.rs"));
}

#[test]
fn test_no_tags_produces_gaps() {
    let dir = TempDir::new().unwrap();
    let manifest_path = minimal_manifest(&dir);

    // Write a Rust file with no trace tags
    let src = r#"
fn main() {
    println!("no tags here");
}
"#;
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), src).unwrap();

    let requirements = manifest::load(&manifest_path).unwrap();
    let tag_hits = scan::scan_repo(dir.path()).unwrap();
    let results = scan::map_requirements(&requirements, &tag_hits);

    let gaps = scan::gaps(&results);
    assert!(
        !gaps.is_empty(),
        "should have gaps when no tags are present"
    );
    assert_eq!(gaps.len(), 3, "all three requirements should be gaps");
}

#[test]
fn test_mapped_tags_cover_requirement() {
    let dir = TempDir::new().unwrap();

    // Manifest with a mapped tag
    let content = r#"requirements:
  - id: V6.1.1
    level: L3
    description: "PII at rest encryption"
    mapped_tags: ["AC-DAT-01", "EC-DAT-01"]
"#;
    let manifest_path = dir.path().join("asvs-manifest.yaml");
    fs::write(&manifest_path, content).unwrap();

    // Test file using EC-DAT-01 tag
    let test_src = r#"
#[trace("EC-DAT-01")]
#[test]
fn test_pii_encryption() {
    assert!(true);
}
"#;
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), test_src).unwrap();

    let requirements = manifest::load(&manifest_path).unwrap();
    let tag_hits = scan::scan_repo(dir.path()).unwrap();
    let results = scan::map_requirements(&requirements, &tag_hits);

    let v611 = results
        .iter()
        .find(|r| r.requirement.id == "V6.1.1")
        .unwrap();
    assert!(
        v611.covered,
        "V6.1.1 should be covered via mapped tag EC-DAT-01"
    );
}

#[test]
fn test_python_pytest_tags() {
    let dir = TempDir::new().unwrap();
    let manifest_path = minimal_manifest(&dir);

    let py_src = r#"
@pytest.mark.trace("ASVS-V2.1.1", "ASVS-V6.2.3")
def test_something():
    pass
"#;
    let test_dir = dir.path().join("tests");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(test_dir.join("test_auth.py"), py_src).unwrap();

    let requirements = manifest::load(&manifest_path).unwrap();
    let tag_hits = scan::scan_repo(dir.path()).unwrap();
    let results = scan::map_requirements(&requirements, &tag_hits);

    let v211 = results
        .iter()
        .find(|r| r.requirement.id == "V2.1.1")
        .unwrap();
    assert!(v211.covered, "V2.1.1 should be covered by Python trace tag");
}
