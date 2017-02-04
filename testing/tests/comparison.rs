extern crate askama;
#[macro_use]
extern crate askama_derive;

use askama::Template;

#[derive(Template)]
#[template(path = "eq.html")]
struct EqTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_eq() {
    let t = EqTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.render(), "tf\n");
}
