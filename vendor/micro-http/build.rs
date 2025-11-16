fn main() {
    #[cfg(not(target_family = "unix"))]
    std::compile_error!("This crate only supports Unix-like targets");
}