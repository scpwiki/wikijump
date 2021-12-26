fn main() {
    // Compile-time build information.
    built::write_built_file().expect("Failed to write build information");

    // Trigger recompilation when a new migration is added.
    println!("cargo:rerun-if-changed=migrations");
}
