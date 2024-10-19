use std::path::Path;

fn main() {
    // Create the protos output directory first AND THEN tell cargo to rerun if
    // the the protos are not there. Apparently, if the (generated) file
    // (src/protos/mod.rs) does not exist, it runs this build script.
    let protos_output_dir = Path::new("src").join("protos");
    std::fs::create_dir_all(&protos_output_dir);
    println!("cargo::rerun-if-changed=src/protos/mod.rs");
    println!("cargo::rerun-if-changed=protos/storage.proto");
    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(&protos_output_dir)
        .input("protos/storage.proto")
        .include("protos")
        .run()
        .expect("failed to codegen")
}
