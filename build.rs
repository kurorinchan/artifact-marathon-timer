use std::path::Path;

fn main() {
    println!("cargo::rerun-if-changed=protos/storage.proto");
    let protos_output_dir = Path::new("src").join("protos");
    std::fs::create_dir_all(&protos_output_dir);
    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(&protos_output_dir)
        .input("protos/storage.proto")
        .include("protos")
        .run()
        .expect("failed to codegen")
}
