#[macro_use]
extern crate askama;

use askama::Template;

macro_rules! hello {
    () => {
        "world"
    };
}

#[derive(Template)]
#[template(path = "rust-macros.html")]
struct RustMacrosTemplate {}

#[test]
fn test_rust_macros() {
    let template = RustMacrosTemplate {};
    assert_eq!("Hello, world!", template.render().unwrap());
}

#[derive(Template)]
#[template(path = "rust-macros-filters.html")]
struct RustMacrosFiltersTemplate<'a> {
    foo: Vec<Bar<'a>>,
}

struct Bar<'a> {
    bar: Option<&'a str>,
    num: Option<i32>,
}

#[test]
fn test_rust_macros_filters() {
    let template = RustMacrosFiltersTemplate {
        foo: vec![
            Bar {
                bar: Some("foo"),
                num: None,
            },
            Bar {
                bar: None,
                num: Some(-1),
            },
            Bar {
                bar: Some("bar"),
                num: Some(12),
            },
        ],
    };
    assert_eq!("foobarFOOBAR112", template.render().unwrap());
}
