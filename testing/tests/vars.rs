#![allow(clippy::useless_let_if_seq)]

use askama::Template;

#[derive(Template)]
#[template(source = "{% let v = s %}{{ v }}", ext = "txt")]
struct LetTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_let() {
    let t = LetTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "let.html")]
struct LetTupleTemplate<'a> {
    s: &'a str,
    t: (&'a str, &'a str),
}

#[test]
fn test_let_tuple() {
    let t = LetTupleTemplate {
        s: "foo",
        t: ("bar", "bazz"),
    };
    assert_eq!(t.render().unwrap(), "foo\nbarbazz");
}

#[derive(Template)]
#[template(path = "let-decl.html")]
struct LetDeclTemplate<'a> {
    cond: bool,
    s: &'a str,
}

#[test]
fn test_let_decl() {
    let t = LetDeclTemplate {
        cond: false,
        s: "bar",
    };
    assert_eq!(t.render().unwrap(), "bar");
}

#[derive(Template)]
#[template(path = "let-shadow.html")]
struct LetShadowTemplate {
    cond: bool,
}

impl LetShadowTemplate {
    fn tuple() -> (i32, i32) {
        (4, 5)
    }
}

#[test]
fn test_let_shadow() {
    let t = LetShadowTemplate { cond: true };
    assert_eq!(t.render().unwrap(), "22-1-33-11-22");

    let t = LetShadowTemplate { cond: false };
    assert_eq!(t.render().unwrap(), "222-1-333-4-5-11-222");
}

#[derive(Template)]
#[template(source = "{% for v in self.0 %}{{ v }}{% endfor %}", ext = "txt")]
struct SelfIterTemplate(Vec<usize>);

#[test]
fn test_self_iter() {
    let t = SelfIterTemplate(vec![1, 2, 3]);
    assert_eq!(t.render().unwrap(), "123");
}

#[derive(Template)]
#[template(
    source = "{% if true %}{% let t = a.unwrap() %}{{ t }}{% endif %}",
    ext = "txt"
)]
struct IfLet {
    a: Option<&'static str>,
}

#[test]
fn test_if_let() {
    let t = IfLet { a: Some("foo") };
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "let-destruct-tuple.html")]
struct LetDestructTupleTemplate {
    abcd: (char, ((char, char), char)),
}

#[test]
fn test_destruct_tuple() {
    let t = LetDestructTupleTemplate {
        abcd: ('w', (('x', 'y'), 'z')),
    };
    assert_eq!(t.render().unwrap(), "wxyz\nwz\nw");
}

#[derive(Template)]
#[template(
    source = "{% let x = 1 %}{% for x in x..=x %}{{ x }}{% endfor %}",
    ext = "txt"
)]
struct DeclRange;

#[test]
fn test_decl_range() {
    let t = DeclRange;
    assert_eq!(t.render().unwrap(), "1");
}

#[derive(Template)]
#[template(
    source = "{% let x %}{% let x = 1 %}{% for x in x..=x %}{{ x }}{% endfor %}",
    ext = "txt"
)]
struct DeclAssignRange;

#[test]
fn test_decl_assign_range() {
    let t = DeclAssignRange;
    assert_eq!(t.render().unwrap(), "1");
}

// This ensures that we do not wrap any call in a reference (which would prevent this template to
// compile).
#[derive(Template)]
#[template(
    source = r#"{% let my_string -%}
{% if a == 1 -%}
{% let my_string = format!("testing {}", true) -%}
{% else if a == 2 -%}
{% let my_string = "testing {}"|format(a) -%}
{% else if a == 3 -%}
{% let my_string = String::from("yop yop") -%}
{% else -%}
{% let my_string = "something else".into() -%}
{% endif %}

{{- my_string }}"#,
    ext = "html"
)]
struct LetWithoutRef {
    a: u32,
}

#[test]
fn test_let_without_ref() {
    let t = LetWithoutRef { a: 1 };
    assert_eq!(t.render().unwrap(), "testing true");
}
