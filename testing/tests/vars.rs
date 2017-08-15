#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "let.html")]
struct LetTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_let() {
    let t = LetTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo");
}
