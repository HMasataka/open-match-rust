fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(&["api/backend.proto"], &[".", "third_party/"])?;
    tonic_build::configure().compile_protos(&["api/evaluator.proto"], &[".", "third_party/"])?;
    tonic_build::configure().compile_protos(&["api/extensions.proto"], &[".", "third_party/"])?;
    tonic_build::configure().compile_protos(&["api/frontend.proto"], &[".", "third_party/"])?;
    tonic_build::configure().compile_protos(&["api/messages.proto"], &[".", "third_party/"])?;
    tonic_build::configure().compile_protos(&["api/query.proto"], &[".", "third_party/"])?;
    Ok(())
}
