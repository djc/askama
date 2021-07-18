use askama::Template;

#[derive(Template)]
#[template(
    source = "{% for v in values %}{{ v }}{% else %}empty{% endfor %}",
    ext = "txt"
)]
struct ForElse<'a> {
    values: &'a [i32],
}

#[test]
fn test_for_else() {
    let t = ForElse { values: &[1, 2, 3] };
    assert_eq!(t.render().unwrap(), "123");

    let t = ForElse { values: &[] };
    assert_eq!(t.render().unwrap(), "empty");
}
