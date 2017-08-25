#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "escaping_filters.html")]
struct TestTemplate {
    strvar: String,
}

#[test]
fn filter_escape() {
    let s = TestTemplate {
        strvar: "my <html> is unsafe & should be escaped".to_string(),
    };
    let expected = "my &lt;html&gt; is unsafe &amp; should be escaped
my &lt;html&gt; is unsafe &amp; should be escaped
my <html> is unsafe & should be escaped
my <html> is unsafe & should be escaped";

    assert_eq!(s.render().unwrap(),
               expected);
}


struct Holder {
    a: String,
}

impl Holder {
    fn gimme(&self) -> &str {
        &self.a
    }
}

#[derive(Template)]
#[template(path = "attr.html")]
struct AttrTemplate {
    inner: Holder,
}

#[test]
fn test_attr() {
    let t = AttrTemplate { inner: Holder { a:  "my <html/> is unsafe & should be escaped".into() }};
    let expected = "my &lt;html/&gt; is unsafe &amp; should be escaped";
    assert_eq!(t.render().unwrap(), expected);
}

#[derive(Template)]
#[template(path = "escaping.html")]
struct MethodTemplate {
    inner: Holder,
}

#[test]
fn test_method() {
    let t = MethodTemplate { inner: Holder { a:  "my <html/> is unsafe & should be escaped!".into() }};
    let expected = "my &lt;html/&gt; is unsafe &amp; should be escaped!";
    assert_eq!(t.render().unwrap(), expected);
}
