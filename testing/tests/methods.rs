use askama::Template;

#[derive(Template)]
#[template(source = "{{ self.get_s() }}", ext = "txt")]
struct SelfMethodTemplate<'a> {
    s: &'a str,
}

impl<'a> SelfMethodTemplate<'a> {
    fn get_s(&self) -> &str {
        self.s
    }
}

#[test]
fn test_self_method() {
    let t = SelfMethodTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(source = "{{ self.get_s() }} {{ t.get_s() }}", ext = "txt")]
struct NestedSelfMethodTemplate<'a> {
    t: SelfMethodTemplate<'a>,
}

impl<'a> NestedSelfMethodTemplate<'a> {
    fn get_s(&self) -> &str {
        "bar"
    }
}

#[test]
fn test_nested() {
    let t = NestedSelfMethodTemplate {
        t: SelfMethodTemplate { s: "foo" },
    };
    assert_eq!(t.render().unwrap(), "bar foo");
}
