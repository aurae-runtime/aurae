use aurae_executables::{Executable, ExecutableName, ExecutableSpec};
use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;
use validation::ValidatedField;

fn main() {
    let mut exe = Executable::new(ExecutableSpec {
        name: ExecutableName::validate(Some("ps-aux".into()), "", None)
            .expect("valid exe name"),
        command: CString::new("ls /proc").expect("valid cstring"),
        description: "List processes".to_string(),
    });

    exe.start().expect("exe start");

    loop {
        println!("Hello, from nested aurae!");
        sleep(Duration::from_secs(60));
    }
}
