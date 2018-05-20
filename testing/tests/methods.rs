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

#[derive(Template)]
#[template(source = "{{ self.get_self() }}", ext = "txt")]
struct SelfTemplate;

impl SelfTemplate {
    fn get_self(&self) -> &Self {
        self
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

// Fails with: thread 'test_self' has overflowed its stack
#[test]
fn test_self() {
    let t = SelfTemplate;
    assert_eq!(t.render().unwrap(), "");
}
