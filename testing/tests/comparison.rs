extern crate askama;
#[macro_use]
extern crate askama_derive;

use askama::Template;

#[derive(Template)]
#[template(path = "comparison.html")]
struct ComparisonTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_comparison() {
    let t = ComparisonTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.render(), "tf\ntf\ntf\ntf\ntf\ntf\n");
}
