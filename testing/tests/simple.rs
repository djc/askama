#![feature(proc_macro)]

#[macro_use]
extern crate askama_derive;
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "simple.html")]
struct VariablesTemplate {
    strvar: String,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables() {
    let s = VariablesTemplate {
        strvar: "foo".to_string(),
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
