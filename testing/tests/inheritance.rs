#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "child.html")]
struct ChildTemplate<'a> {
    _parent: BaseTemplate<'a>,
}

#[test]
fn test_use_base_directly() {
    let t = BaseTemplate { title: "Foo" };
    assert_eq!(t.render().unwrap(), "Foo\n\nFoo\nCopyright 2017");
}

#[test]
fn test_simple_extends() {
    let t = ChildTemplate { _parent: BaseTemplate { title: "Bar" } };
    assert_eq!(
        t.render().unwrap(),
        "Bar\n(Bar) Content goes here\nFoo\nCopyright 2017"
    );
}


pub mod parent {
    use askama::Template;
    #[derive(Template)]
    #[template(path = "base.html")]
    pub struct BaseTemplate<'a> {
        pub title: &'a str,
    }
}

pub mod child {
    use askama::Template;
    use super::parent::*;
    #[derive(Template)]
    #[template(path = "child.html")]
    pub struct ChildTemplate<'a> {
        pub _parent: BaseTemplate<'a>,
    }
}

#[test]
fn test_different_module() {
    let t = child::ChildTemplate { _parent: parent::BaseTemplate { title: "a" } };
    assert_eq!(
        t.render().unwrap(),
        "a\n(a) Content goes here\nFoo\nCopyright 2017"
    );
}



#[derive(Template)]
#[template(path = "nested-base.html")]
struct NestedBaseTemplate {}

#[derive(Template)]
#[template(path = "nested-child.html")]
struct NestedChildTemplate {
    _parent: NestedBaseTemplate,
}

#[test]
fn test_nested_blocks() {
    let t = NestedChildTemplate { _parent: NestedBaseTemplate {} };
    assert_eq!(t.render().unwrap(), "\ndurpy\n");
}
