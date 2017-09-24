#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "import.html")]
struct ImportTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_import() {
    let t = ImportTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo foo foo");
}
