extern crate built;

use std::env;

fn main() {
    // Generate build information
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", &profile);
    }

    built::write_built_file().expect("Failed to compile build information!");

    // Set openssl library
    if env::var("CARGO_CFG_UNIX").is_ok() {
        println!("cargo:rustc-flags=-L /usr/lib/openssl-1.0");
    }
}
