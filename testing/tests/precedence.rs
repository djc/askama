#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "precedence.html")]
struct PrecedenceTemplate {
}

#[test]
fn test_precedence() {
    let t = PrecedenceTemplate { };
    assert_eq!(t.render().unwrap(), "6".repeat(7));
}
