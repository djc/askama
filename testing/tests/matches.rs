#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "match-opt.html")]
struct MatchOptTemplate<'a> {
    item: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "match-opt.html")]
struct MatchOptRefTemplate<'a> {
    item: &'a Option<&'a str>,
}


#[test]
fn test_match_option() {
    let s = MatchOptTemplate { item: Some("foo") };
    assert_eq!(s.render().unwrap(), "\nFound literal foo\n");

    let s = MatchOptTemplate { item: Some("bar") };
    assert_eq!(s.render().unwrap(), "\n\nFound bar\n");

    let s = MatchOptTemplate { item: None };
    assert_eq!(s.render().unwrap(), "\n\nNot Found\n");
}

#[test]
fn test_match_ref_deref() {
    let s = MatchOptRefTemplate { item: &Some("foo") };
    assert_eq!(s.render().unwrap(), "\nFound literal foo\n");
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

#[derive(Template)]
#[template(path = "match-literal-num.html")]
struct MatchLitNumTemplate {
    item: u32,
}

#[test]
fn test_match_literal_num() {
    let s = MatchLitNumTemplate { item: 42 };
    assert_eq!(s.render().unwrap(), "\nFound answer to everything\n");

    let s = MatchLitNumTemplate { item: 23 };
    assert_eq!(s.render().unwrap(), "\n\nElse found 23\n");
}
