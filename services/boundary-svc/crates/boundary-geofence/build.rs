fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile proto relative to the workspace root (services/boundary-svc/).
    // When building from services/boundary-svc/ the CARGO_MANIFEST_DIR is the
    // crate directory, so we walk up two levels to reach services/boundary-svc/.
    let proto_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&[proto_root.join("geofence.proto")], &[&proto_root])?;
    Ok(())
}
