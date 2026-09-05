// NBIS v5.0.0 — real NIST source compiled from vendor/
// Compiles minutiae extraction (mindtct) + matching (bozorth3) + quality (nfiq)
use std::env;
use std::path::PathBuf;

fn main() {
    let vendor = PathBuf::from("vendor/nbis");

    // Collect all NBIS C sources
    let mut cc = cc::Build::new();
    cc.include(&vendor);
    cc.include(vendor.join("bozorth3"));
    cc.include(vendor.join("mindtct"));
    cc.include(vendor.join("nfiq"));
    cc.include(vendor.join("commonnbis").join("include"));
    cc.define("NBIS_VERSION", "\"5.0.0\"");
    cc.define("__NBIS_LINUX__", None);

    // Bozorth3 matching library
    for file in std::fs::read_dir(vendor.join("bozorth3")).unwrap() {
        let entry = file.unwrap();
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.ends_with(".c") && s != "Makefile" {
            cc.file(entry.path());
        }
    }

    // Mindtct extraction library
    for file in std::fs::read_dir(vendor.join("mindtct")).unwrap() {
        let entry = file.unwrap();
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.ends_with(".c") && s != "Makefile" {
            cc.file(entry.path());
        }
    }

    // Common NBIS library (util, ioutil, fet)
    for subdir in ["util", "ioutil", "fet"] {
        for entry in std::fs::read_dir(vendor.join("commonnbis").join(subdir)).unwrap() {
            let e = entry.unwrap();
            if e.path().extension().map_or(false, |ext| ext == "c") {
                cc.file(e.path());
            }
        }
    }

    // NFIQ quality library
    for file in std::fs::read_dir(vendor.join("nfiq")).unwrap() {
        let entry = file.unwrap();
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.ends_with(".c") && s != "Makefile" {
            cc.file(entry.path());
        }
    }

    // NBIS uses some POSIX functions; link math
    cc.flag_if_supported("-std=c99");
    cc.flag_if_supported("-O2");
    cc.compile("nbis_core");

    // Print for verification
    println!("cargo:rustc-link-lib=static=nbis_core");
    println!("cargo:rerun-if-changed=vendor/nbis");
}
