use trybuild::TestCases;

#[cfg_attr(not(windows), test)]
fn ui() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
