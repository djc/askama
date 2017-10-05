#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "match.html")]
struct MatchTemplate<'a> {
    item: Option<&'a str>,
}

#[test]
fn test_match_option() {
    let s = MatchTemplate {
        item: Some("foo"),
    };
    assert_eq!(s.render().unwrap(), "\nFound foo\n");
}
