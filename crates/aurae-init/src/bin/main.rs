use std::thread::sleep;
use std::time::Duration;

fn main() {
    loop {
        println!("Hello, from nested aurae!");
        sleep(Duration::from_secs(5));
    }
}
