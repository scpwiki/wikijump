extern crate built;

use std::env;

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", &profile);
    }

    built::write_built_file().expect("Failed to compile build information!");
}
