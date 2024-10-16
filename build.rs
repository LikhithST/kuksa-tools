fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &[
                "proto/kuksa/val/v1/val.proto",
                "proto/kuksa/val/v1/types.proto",
            ], // Compile both val.proto and types.proto
            &["proto"], // The base directory containing the .proto files
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));

    println!("OUT_DIR = {:?}", std::env::var("OUT_DIR"));
    Ok(())
}
