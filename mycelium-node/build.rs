fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/grpc/proto")
        .compile_protos(
            &[
                "../proto/grpc/mycelium_service.proto",
                "../proto/grpc/copy.proto",
                "../proto/grpc/echo.proto",
                "../proto/grpc/search.proto",
                "../proto/grpc/common.proto",
            ],
            &["../proto/grpc/"],
        )?;
    Ok(())
}
