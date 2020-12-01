use askama::Template;

#[derive(Template)]
#[template(path = "if-coerce.html")]
struct IfCoerceTemplate {
    t: bool,
    f: bool,
}

#[test]
fn test_coerce() {
    let t = IfCoerceTemplate { t: true, f: false };
    assert_eq!(t.render().unwrap(), "ftftfttftelseifelseif");
}
