extern crate askama;
#[macro_use]
extern crate askama_derive;

use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate { }

#[derive(Template)]
#[template(path = "child.html")]
struct ChildTemplate {
    _parent: BaseTemplate,
}

#[test]
fn test_use_base_directly() {
    let t = BaseTemplate {};
    assert_eq!(t.render(), "\nCopyright 2017\n");
}

#[test]
fn test_simple_extends() {
    let t = ChildTemplate { _parent: BaseTemplate {} };
    assert_eq!(t.render(), "Content goes here\nCopyright 2017\n");
}
