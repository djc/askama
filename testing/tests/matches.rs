#![allow(clippy::blacklisted_name)]

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
    assert_eq!(s.render().unwrap(), "\nFound bar\n");

    let s = MatchOptTemplate { item: None };
    assert_eq!(s.render().unwrap(), "\nNot Found\n");
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
    assert_eq!(s.render().unwrap(), "\nFound literal bar\n");

    let s = MatchLitTemplate { item: "qux" };
    assert_eq!(s.render().unwrap(), "\nElse found qux\n");
}

#[derive(Template)]
#[template(path = "match-literal-char.html")]
struct MatchLitCharTemplate {
    item: char,
}

#[test]
fn test_match_literal_char() {
    let s = MatchLitCharTemplate { item: 'b' };
    assert_eq!(s.render().unwrap(), "\nFound literal b\n");

    let s = MatchLitCharTemplate { item: 'c' };
    assert_eq!(s.render().unwrap(), "\nElse found c\n");
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
    assert_eq!(s.render().unwrap(), "\nElse found 23\n");
}

#[allow(dead_code)]
enum Color {
    Rgb { r: u32, g: u32, b: u32 },
    GrayScale(u32),
    Cmyk(u32, u32, u32, u32),
}

#[derive(Template)]
#[template(path = "match-custom-enum.html")]
struct MatchCustomEnumTemplate {
    color: Color,
}

#[test]
fn test_match_custom_enum() {
    let s = MatchCustomEnumTemplate {
        color: Color::Rgb {
            r: 160,
            g: 0,
            b: 255,
        },
    };
    assert_eq!(s.render().unwrap(), "\nColorful: #A000FF\n");
}

#[derive(Template)]
#[template(path = "match-no-ws.html")]
struct MatchNoWhitespace {
    foo: Option<usize>,
}

#[test]
fn test_match_no_whitespace() {
    let s = MatchNoWhitespace { foo: Some(1) };
    assert_eq!(s.render().unwrap(), "1");
}
