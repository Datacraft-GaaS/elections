fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if arch == "wasm32" {
        return;
    }

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let header_out = std::path::PathBuf::from(&out_dir).join("libeel-verifier.h");

    if let Ok(bindings) = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(
            cbindgen::Config::from_file(std::path::Path::new(&crate_dir).join("cbindgen.toml"))
                .unwrap_or_default(),
        )
        .generate()
    {
        bindings.write_to_file(&header_out);
        let dest = std::path::Path::new(&crate_dir).join("libeel-verifier.h");
        let _ = std::fs::copy(&header_out, dest);
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
