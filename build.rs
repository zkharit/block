fn main () -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
    .compile(
        &[
            "proto/ping.proto",
            "proto/transaction.proto",
            "proto/block.proto"
        ],
        &["proto/"],
    )?;
    Ok(())
}