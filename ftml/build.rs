extern crate built;

use cbindgen::Language;
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Generate build information
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", &profile);
    }

    built::write_built_file().expect("Failed to compile build information!");

    // Generate C bindings for FFI
    let target_path = {
        let target_dir = env::var("OUT_DIR").unwrap();

        // This takes the form '/path/to/repo/target/release/build/ftml-0000000000000000/out'
        // We want '/path/to/repo/target/release/ftml.h'
        let mut path = PathBuf::from(target_dir);
        path.pop();
        path.pop();
        path.pop();
        path.push("ftml.h");
        path
    };

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(Language::C)
        .with_no_includes()
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(&target_path);
}
