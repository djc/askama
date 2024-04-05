#![cfg(not(windows))]

use std::os::unix::fs::symlink;
use std::path::Path;
use trybuild::TestCases;
use version_check as rustc;

#[test]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/*.rs");

    if rustc::is_min_version("1.58").unwrap() {
        t.compile_fail("tests/ui/since_1.58/*.rs");
    }

    if rustc::is_max_version("1.57").unwrap() {
        t.compile_fail("tests/ui/before_1.58/*.rs");
    }

    if rustc::is_min_version("1.54").unwrap() && rustc::is_max_version("1.57").unwrap() {
        t.compile_fail("tests/ui/1.54_to_1.57/*.rs");
    }

    if rustc::is_max_version("1.53").unwrap() {
        t.compile_fail("tests/ui/before_1.54/*.rs");
    }

    // To be able to use existing templates, we create a link to the `templates` folder.
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(manifest_dir) => manifest_dir,
        Err(_) => panic!("you need to run tests with `cargo`"),
    };
    let target = Path::new(&manifest_dir).join("../target/tests/trybuild/askama_testing");
    if !target.exists() {
        if let Err(err) = std::fs::create_dir_all(&target) {
            panic!("failed to create folder `{}`: {err:?}", target.display());
        }
    }
    let target = target.canonicalize().unwrap().join("templates");
    if target.exists() {
        return;
    }
    let original = Path::new(&manifest_dir).join("templates");
    if symlink(&original, &target).is_err() {
        panic!(
            "failed to create to create link on `{}` as `{}`",
            original.display(),
            target.display()
        );
    }
}
