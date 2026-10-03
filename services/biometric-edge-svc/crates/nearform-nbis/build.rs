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

// NBIS v5.0.0 — real NIST source compiled from vendor/
// Compiles minutiae extraction (mindtct) + matching (bozorth3) + quality (nfiq)
// Layout: verbatim upstream subtrees, see vendor/README.provenance.
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let vendor = PathBuf::from("vendor/nbis");

    // an2k.h is generated at build time (mirrors setup.sh):
    //   sed 's,SED_INSTALL_DATA_DIR_STRING,<dir>,' an2k.h.src > an2k.h
    // Generated into OUT_DIR so the vendored tree stays pristine.
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR set by cargo"));
    let install_data = out_dir.join("nbis_data");
    std::fs::create_dir_all(&install_data).expect("create nbis data dir");
    let template = std::fs::read_to_string(vendor.join("an2k").join("include").join("an2k.h.src"))
        .expect("vendored an2k.h.src present (see vendor/README.provenance)");
    let generated = template.replace(
        "SED_INSTALL_DATA_DIR_STRING",
        &install_data.to_string_lossy(),
    );
    std::fs::write(out_dir.join("an2k.h"), generated).expect("write generated an2k.h");

    // Collect a directory of .c files (non-recursive; NBIS keeps each
    // library's sources flat under src/lib/<name>/). The directory itself
    // joins the include path because NBIS headers live beside sources.
    // `exclude_prefixes` skips tool mains (cjpeg/djpeg/...) that must not
    // land in the static library.
    fn add_dir(cc: &mut cc::Build, dir: &Path, exclude_prefixes: &[&str]) {
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
            panic!(
                "missing vendored NBIS dir {}: {} (see vendor/README.provenance)",
                dir.display(),
                e
            )
        });
        for file in entries {
            let entry = file.unwrap();
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.extension().is_some_and(|ext| ext == "c")
                && name != "Makefile"
                && !exclude_prefixes.iter().any(|p| name.starts_with(p))
            {
                cc.file(path);
            }
        }
    }

    /// No exclusions (pure libraries).
    fn add_lib(cc: &mut cc::Build, vendor: &Path, parts: &[&str]) {
        let mut dir = vendor.to_path_buf();
        for p in parts {
            dir = dir.join(p);
        }
        add_dir(cc, &dir, &[]);
    }

    /// Filtered variant: `keep(name)` decides per `.c` file.
    fn add_dir_filter(cc: &mut cc::Build, dir: &Path, keep: &dyn Fn(&str) -> bool) {
        let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
            panic!(
                "missing vendored NBIS dir {}: {} (see vendor/README.provenance)",
                dir.display(),
                e
            )
        });
        for file in entries {
            let entry = file.unwrap();
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.extension().is_some_and(|ext| ext == "c") && keep(&name) {
                cc.file(path);
            }
        }
    }

    let mut cc = cc::Build::new();
    // NBIS headers: per-library include/ dirs + shared commonnbis/include
    // + imgtools/include (ioutil.h and friends live there, not in
    // commonnbis) + OUT_DIR (generated an2k.h).
    for inc in ["bozorth3", "mindtct", "nfiq", "an2k"] {
        cc.include(vendor.join(inc).join("include"));
    }
    cc.include(vendor.join("commonnbis").join("include"));
    cc.include(vendor.join("imgtools").join("include"));
    cc.include(&out_dir);
    // Every compiled lib dir joins the include path (headers beside sources).
    for lib in [
        ("bozorth3", "bozorth3"),
        ("mindtct", "mindtct"),
        ("nfiq", "nfiq"),
        ("an2k", "an2k"),
        ("commonnbis", "fet"),
        ("commonnbis", "ioutil"),
        ("commonnbis", "util"),
        ("ijg", "jpegb"),
    ] {
        cc.include(vendor.join(lib.0).join("src").join("lib").join(lib.1));
    }
    cc.include(vendor.join("commonnbis").join("include"));
    cc.define("NBIS_VERSION", "\"5.0.0\"");
    cc.define("__NBIS_LINUX__", None);

    // Bozorth3 matching library.
    add_lib(&mut cc, &vendor, &["bozorth3", "src", "lib", "bozorth3"]);
    // AN2K record I/O (required by mindtct headers). decode.c/dec_png.c
    // need system libpng, so they are excluded — nothing in the compiled
    // set references PNG decode (linker verified).
    add_dir(
        &mut cc,
        &vendor.join("an2k").join("src").join("lib").join("an2k"),
        &["decode", "dec_png"],
    );
    // Mindtct extraction library.
    add_lib(&mut cc, &vendor, &["mindtct", "src", "lib", "mindtct"]);
    // NFIQ quality library. NOTE: nfiq.c itself is excluded: it calls
    // runmlp2(), whose implementation is absent upstream (declared in
    // nfiq/include/mlp.h, defined nowhere in the mirror). The remaining
    // nfiq sources (globals, readers, znorm) compile for future binding;
    // quality screening uses the documented estimator in src/nfiq.rs.
    add_dir_filter(
        &mut cc,
        &vendor.join("nfiq").join("src").join("lib").join("nfiq"),
        &|name: &str| name != "nfiq.c",
    );
    // Tool-main globals the libraries expect (see vendor/shim/nbis_shims.c).
    cc.file(vendor.join("..").join("shim").join("nbis_shims.c"));
    // Common NBIS libraries (util, ioutil, fet).
    for subdir in ["util", "ioutil", "fet"] {
        add_lib(&mut cc, &vendor, &["commonnbis", "src", "lib", subdir]);
    }
    // IJG JPEG (imgtools headers need jpeglib.h; tool mains excluded).
    // Memory backends: only jmemnobs (jmemdos/jmemmac are DOS/classic-Mac
    // and do not compile elsewhere; jmemansi/jmemname need OS tmpfile
    // semantics we do not rely on — jmemmgr dispatches to jmemnobs).
    add_dir(
        &mut cc,
        &vendor.join("ijg").join("src").join("lib").join("jpegb"),
        &[
            "cjpeg", "djpeg", "ckconfig", "example", "ansi2knr", "coderule", "cdjpeg", "rdjpgcom",
            "wrjpgcom", "rdswitch", "jpegtran", "transupp", "jmemdos", "jmemmac", "jmemansi",
            "jmemname",
        ],
    );
    cc.include(vendor.join("ijg").join("src").join("lib").join("jpegb"));

    // NBIS uses some POSIX functions; link math.
    cc.flag_if_supported("-std=c99");
    cc.flag_if_supported("-O2");
    cc.compile("nbis_core");

    // Print for verification
    println!("cargo:rustc-link-lib=static=nbis_core");
    println!("cargo:rerun-if-changed=vendor/nbis");
    println!("cargo:rerun-if-changed=vendor/shim");
}
