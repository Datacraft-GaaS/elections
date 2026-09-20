fn main() {
    // On macOS with OpenSSL 3.x, librdkafka (ssl feature) references ENGINE_* symbols
    // that were removed from OpenSSL 3.x. Provide no-op stubs so dev builds link.
    // Linux production builds use distro OpenSSL with compat layers.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        let stub_src = std::path::Path::new("src/engine_stubs.c");
        if stub_src.exists() {
            cc::Build::new().file(stub_src).compile("engine_stubs");
            println!("cargo:rustc-link-lib=static=engine_stubs");
        }
    }
}
