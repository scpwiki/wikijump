use std::env;

fn main() {
    // Set openssl library
    if env::var("CARGO_CFG_UNIX").is_ok() {
        println!("cargo:rustc-flags=-L /usr/lib/openssl-1.0");
    }
}
