use std::process::Command;

/// Will run `buf generate` when a file in the api directory changes.
fn main() {
    println!("cargo:rerun-if-changed=../../api");

    Command::new("buf")
        .args(["--version"])
        .output()
        .expect("Warning: buf is not installed! Please install the 'buf' command line tool: https://docs.buf.build/installation");

    Command::new("buf")
        .args([
            "generate",
            "../../api",
            "-v",
            "--template",
            "../../buf.gen.yaml",
            "-o",
            "../../",
        ])
        .output()
        .expect("`buf generate` failed");
}
