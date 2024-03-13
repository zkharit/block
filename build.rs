fn main () -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/ping.proto")?;
    tonic_build::compile_protos("proto/transaction.proto")?;
    Ok(())
}