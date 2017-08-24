#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "filters.html")]
struct TestTemplate {
    strvar: String,
}

#[test]
fn filter_escape() {
    let s = TestTemplate {
        strvar: "my <html> is unsafe & should be escaped".to_string(),
    };
    assert_eq!(s.render().unwrap(),
               "my &lt;html&gt; is unsafe &amp; should be escaped");
}


#[derive(Template)]
#[template(path = "format.html")]
struct FormatTemplate<'a> {
    var: &'a str,
}

#[test]
fn filter_format() {
    let t = FormatTemplate { var: "formatted" };
    assert_eq!(t.render().unwrap(), "\"formatted\"");
}


#[derive(Template)]
#[template(path = "length.html")]
struct LengthTemplate<'a> {
    var: &'a str,
}

#[test]
fn filter_length() {
    let t = LengthTemplate { var: "length" };
    let expected = "6, 6";
    assert_eq!(t.render().unwrap(), expected);
}
