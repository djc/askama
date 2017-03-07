#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "for.html")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["A", "alfa", "1"],
    };
    assert_eq!(s.render(), "0. A\n1. alfa\n2. 1\n");
}
