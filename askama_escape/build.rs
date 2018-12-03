extern crate version_check;

fn main() {
    match version_check::is_nightly() {
        Some(true) => {
            println!("cargo:rustc-cfg=askama_nightly");
        }
        Some(false) => (),
        None => (),
    };
}
