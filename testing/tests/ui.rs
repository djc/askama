use trybuild::TestCases;
use version_check as rustc;

#[cfg_attr(not(windows), test)]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/*.rs");

    if rustc::is_min_version("1.54").unwrap() {
        t.compile_fail("tests/ui/since_1_54/*.rs");
    } else {
        t.compile_fail("tests/ui/before_1_54/*.rs");
    }
}
