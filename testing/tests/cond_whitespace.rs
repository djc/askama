use askama::Template;

#[derive(askama::Template, Default)]
#[template(path = "cond-whitespace.txt")]
struct CondWhitespace {
    show_preamble: bool,
}

#[test]
fn test_cond_whitespace() {
    let template = CondWhitespace::default();
    assert_eq!(template.render().unwrap(), "introduction\n\nconclusion");
}
