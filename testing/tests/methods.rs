#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(source = "{{ self.get_s() }}", ext = "txt")]
struct MethodTemplate<'a> {
    s: &'a str,
}

impl<'a> MethodTemplate<'a> {
    fn get_s(&self) -> &str {
        self.s
    }
}

#[derive(Template)]
#[template(source = "{{ self.get_s() }} {{ t.get_s() }}", ext = "txt")]
struct NestedMethodTemplate<'a> {
    t: MethodTemplate<'a>,
}

impl<'a> NestedMethodTemplate<'a> {
    fn get_s(&self) -> &str {
        "bar"
    }
}

#[test]
fn test_method() {
    let t = MethodTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo");
}

#[test]
fn test_nested() {
    let t = NestedMethodTemplate {
        t: MethodTemplate { s: "foo" },
    };
    assert_eq!(t.render().unwrap(), "bar foo");
}
