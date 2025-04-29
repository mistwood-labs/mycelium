fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &["../proto/mycelium.proto"];
    let proto_include = &["../proto"];

    prost_build::compile_protos(proto_files, proto_include)?;

    println!("cargo:rerun-if-changed=../proto/mycelium.proto");

    Ok(())
}
