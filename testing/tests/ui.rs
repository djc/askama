use trybuild::TestCases;
use version_check as rustc;

#[cfg_attr(not(windows), test)]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/*.rs");

    if rustc::is_min_version("1.58").unwrap() {
        t.compile_fail("tests/ui/since_1.58/*.rs");
    }

    if rustc::is_max_version("1.57").unwrap() {
        t.compile_fail("tests/ui/before_1.58/*.rs");
    }

    if rustc::is_min_version("1.54").unwrap() {
        t.compile_fail("tests/ui/since_1.54/*.rs");
    }

    if rustc::is_min_version("1.54").unwrap() && rustc::is_max_version("1.57").unwrap() {
        t.compile_fail("tests/ui/1.54_to_1.57/*.rs");
    }

    if rustc::is_max_version("1.53").unwrap() {
        t.compile_fail("tests/ui/before_1.54/*.rs");
    }
}
