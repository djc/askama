fn main() {
    // This build script only exists so that OUT_DIR is set.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
}
