#![deny(warnings)]

use std::os::unix::io::AsRawFd;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Need exactly one parameter: path to a criu binary");
        std::process::exit(1);
    }

    let criu_bin_path = args[1].clone();
    if !Path::new(&criu_bin_path).is_file() {
        println!("Invalid path to a criu binary");
        std::process::exit(1);
    }

    let mut criu = rust_criu::Criu::new().unwrap();
    match criu.get_criu_version() {
        Ok(version) => println!("Version from CRIU found in $PATH: {}", version),
        Err(e) => println!("{:#?}", e),
    };

    criu = rust_criu::Criu::new_with_criu_path(criu_bin_path).unwrap();
    match criu.get_criu_version() {
        Ok(version) => println!("Version from {}: {}", args[1], version),
        Err(e) => println!("{:#?}", e),
    };

    let pid = match std::process::Command::new("test/piggie").output() {
        Ok(p) => String::from_utf8_lossy(&p.stdout).parse().unwrap_or(0),
        Err(e) => panic!("Starting test process failed ({:#?})", e),
    };

    criu.set_pid(pid);

    if let Err(e) = std::fs::create_dir("test/images") {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            panic!(
                "Creating image directory 'test/images' failed with {:#?}",
                e
            );
        }
    }

    let directory = std::fs::File::open(String::from("test/images")).unwrap();
    criu.set_images_dir_fd(directory.as_raw_fd());
    // Using a non-default log_file name to be able to check if it has been created.
    criu.set_log_file("dumppp.log".to_string());
    criu.set_log_level(4);
    println!("Dumping PID {}", pid);
    if let Err(e) = criu.dump() {
        panic!("Dumping process failed with {:#?}", e);
    }

    if !std::path::Path::new("test/images/dumppp.log").exists() {
        panic!("Error: Expected log file 'test/images/dumppp.log' missing.");
    }

    // Need to set all values again as everything is being cleared after success.
    criu.set_images_dir_fd(directory.as_raw_fd());
    criu.set_log_level(4);
    criu.set_log_file("restoreee.log".to_string());
    println!("Restoring PID {}", pid);
    if let Err(e) = criu.restore() {
        panic!("Restoring process failed with {:#?}", e);
    }
    if !std::path::Path::new("test/images/restoreee.log").exists() {
        panic!("Error: Expected log file 'test/images/restoreee.log' missing.");
    }

    println!("Cleaning up");
    if let Err(e) = std::fs::remove_dir_all("test/images") {
        panic!(
            "Removing image directory 'test/images' failed with {:#?}",
            e
        );
    }
}
