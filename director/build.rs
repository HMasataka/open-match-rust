fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../api/backend.proto", "../api/matchfunction.proto"],
        &["..", "../third_party/"],
    )?;
    Ok(())
}
