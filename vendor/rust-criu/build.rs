extern crate protobuf_codegen;

fn main() {
    if std::env::var_os("GENERATE_PROTOBUF").is_some() {
        protobuf_codegen::Codegen::new()
            .includes(["proto"])
            .input("proto/rpc.proto")
            .out_dir("src/rust_criu_protobuf")
            .run_from_script();
    }

    if std::env::var_os("GENERATE_TEST_PROCESS").is_some() {
        std::process::Command::new("gcc")
            .args(["test/piggie.c", "-o", "test/piggie"])
            .status()
            .unwrap();
    }
    println!("cargo:rerun-if-changed=test/piggie.c");
    println!("cargo:rerun-if-changed=proto/rpc.proto");
    println!("cargo:rerun-if-env-changed=GENERATE_PROTOBUF");
    println!("cargo:rerun-if-env-changed=GENERATE_TEST_PROCESS");
}
