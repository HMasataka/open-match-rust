fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile_protos(&["../api/matchfunction.proto"], &["..", "../third_party/"])?;
    Ok(())
}
