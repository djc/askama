#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "match-opt.html")]
struct MatchOptTemplate<'a> {
    item: Option<&'a str>,
}

#[test]
fn test_match_option() {
    let s = MatchOptTemplate { item: Some("foo") };
    assert_eq!(s.render().unwrap(), "\n\nFound literal foo\n");

    let s = MatchOptTemplate { item: Some("bar") };
    assert_eq!(s.render().unwrap(), "\n\nFound bar\n");

    let s = MatchOptTemplate { item: None };
    assert_eq!(s.render().unwrap(), "\n\nNot Found\n");
}


#[derive(Template)]
#[template(path = "match-literal.html")]
struct MatchLitTemplate<'a> {
    item: &'a str,
}

#[test]
fn test_match_literal() {
    let s = MatchLitTemplate { item: "bar" };
    assert_eq!(s.render().unwrap(), "\n\nFound literal bar\n");

    let s = MatchLitTemplate { item: "qux" };
    assert_eq!(s.render().unwrap(), "\n\nElse found qux\n");
}
