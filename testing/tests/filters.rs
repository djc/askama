#[macro_use]
extern crate askama_derive;
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
    assert_eq!(s.render(),
               "my &lt;html&gt; is unsafe &amp; should be escaped\n");
}
