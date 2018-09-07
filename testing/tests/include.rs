#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "include.html")]
struct IncludeTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_include() {
    let strs = vec!["foo", "bar"];
    let s = IncludeTemplate { strs: &strs };
    assert_eq!(s.render().unwrap(), "\n  INCLUDED: foo\n  INCLUDED: bar")
}

#[derive(Template)]
#[template(path = "deep-include-parent.html")]
struct DeepIncludeTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_deep_include() {
    let strs = vec!["foo", "bar"];
    let s = DeepIncludeTemplate { strs: &strs };
    assert_eq!(s.render().unwrap(), "\n  INCLUDED: foo\n  INCLUDED: bar")
}


#[derive(Template)]
#[template(path = "deep-include-for-parent.html")]
struct DeepForAtIncludeTemplate<'a> {
    strss: &'a [[&'a str; 2]],
}

#[test]
fn test_deep_for_at_include() {
    let strs = vec![["foo", "bar"], ["bar", "foo"]];
    let s = DeepForAtIncludeTemplate { strss: &strs };
    assert_eq!(
        s.render().unwrap(), "\n  INCLUDED: foo\n  INCLUDED: bar\n  INCLUDED: bar\n  INCLUDED: foo"
    )
}
