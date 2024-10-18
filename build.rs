fn main() {
    println!("cargo::rerun-if-changed=protos/storage.proto");
    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir("src/protos")
        .input("protos/storage.proto")
        .include("protos")
        .run()
        .expect("failed to codegen")
}
