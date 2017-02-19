#[macro_use]
extern crate askama_derive;
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "simple.html")]
struct VariablesTemplate<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables() {
    let s = VariablesTemplate {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(s.render(), "hello world, foo\n\
                            with number: 42\n\
                            Iñtërnâtiônàlizætiøn is important\n\
                            in vars too: Iñtërnâtiônàlizætiøn\n");
}


#[derive(Template)]
#[template(path = "if.html")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let s = IfTemplate { cond: true };
    assert_eq!(s.render(), "true\n");
}


#[derive(Template)]
#[template(path = "else.html")]
struct ElseTemplate {
    cond: bool,
}

#[test]
fn test_else() {
    let s = ElseTemplate { cond: false };
    assert_eq!(s.render(), "false\n");
}


#[derive(Template)]
#[template(path = "else-if.html")]
struct ElseIfTemplate {
    cond: bool,
    check: bool,
}

#[test]
fn test_else_if() {
    let s = ElseIfTemplate { cond: false, check: true };
    assert_eq!(s.render(), "checked\n");
}


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
    assert_eq!(s.render(), "0. A\n1. alfa\n2. 1\n\n");
}


#[derive(Template)]
#[template(path = "literals.html")]
struct LiteralsTemplate {}

#[test]
fn test_literals() {
    let s = LiteralsTemplate {};
    assert_eq!(s.render(), "a\n");
}


struct Holder {
    a: usize,
}

#[derive(Template)]
#[template(path = "attr.html")]
struct AttrTemplate {
    inner: Holder,
}

#[test]
fn test_attr() {
    let t = AttrTemplate { inner: Holder { a: 5 } };
    assert_eq!(t.render(), "5\n");
}


#[derive(Template)]
#[template(path = "option.html")]
struct OptionTemplate<'a> {
    var: Option<&'a str>,
}

#[test]
fn test_option() {
    let some = OptionTemplate { var: Some("foo") };
    assert_eq!(some.render(), "some: foo\n");
    let none = OptionTemplate { var: None };
    assert_eq!(none.render(), "none\n");
}
