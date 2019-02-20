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
#[template(
    source = "{% for v in self.0 %}{{ v }}{% endfor %}",
    ext = "txt",
    print = "code"
)]
struct SelfIterTemplate(Vec<usize>);

#[test]
fn test_self_iter() {
    let t = SelfIterTemplate(vec![1, 2, 3]);
    assert_eq!(t.render().unwrap(), "123");
}
