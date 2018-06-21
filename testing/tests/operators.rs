#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "compare.html")]
struct CompareTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_compare() {
    let t = CompareTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.render().unwrap(), "tf\ntf\ntf\ntf\ntf\ntf");
}

#[derive(Template)]
#[template(path = "operators.html")]
struct OperatorsTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_operators() {
    let t = OperatorsTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.render().unwrap(), "muldivmodaddrshlshbandbxorborandor");
}

#[derive(Template)]
#[template(path = "precedence.html")]
struct PrecedenceTemplate {}

#[test]
fn test_precedence() {
    let t = PrecedenceTemplate {};
    assert_eq!(t.render().unwrap(), "6".repeat(7));
}
