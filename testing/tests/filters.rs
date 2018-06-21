#[macro_use]
extern crate askama;
#[macro_use]
extern crate serde_json;

use askama::Template;
use serde_json::Value;

#[derive(Template)]
#[template(path = "filters.html")]
struct TestTemplate {
    strvar: String,
}

#[test]
fn filter_escape() {
    let s = TestTemplate {
        strvar: "// my <html> is \"unsafe\" & should be 'escaped'".to_string(),
    };
    assert_eq!(
        s.render().unwrap(),
        "&#x2f;&#x2f; my &lt;html&gt; is &quot;unsafe&quot; &amp; \
         should be &#x27;escaped&#x27;"
    );
}

#[derive(Template)]
#[template(path = "format.html", escape = "none")]
struct FormatTemplate<'a> {
    var: &'a str,
}

#[test]
fn filter_format() {
    let t = FormatTemplate { var: "formatted" };
    assert_eq!(t.render().unwrap(), "\"formatted\"");
}

#[derive(Template)]
#[template(source = "{{ s|myfilter }}", ext = "txt")]
struct MyFilterTemplate<'a> {
    s: &'a str,
}

mod filters {
    pub fn myfilter(s: &str) -> ::askama::Result<String> {
        Ok(s.replace("oo", "aa").to_string())
    }
    // for test_nested_filter_ref
    pub fn mytrim(s: &::std::fmt::Display) -> ::askama::Result<String> {
        let s = format!("{}", s);
        Ok(s.trim().to_owned())
    }
}

#[test]
fn test_my_filter() {
    let t = MyFilterTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "faa");
}

#[derive(Template)]
#[template(path = "filters_join.html")]
struct JoinTemplate<'a> {
    s: &'a [&'a str],
}

#[test]
fn test_join() {
    let t = JoinTemplate {
        s: &["foo", "bar", "bazz"],
    };
    assert_eq!(t.render().unwrap(), "foo, bar, bazz");
}

#[derive(Template)]
#[template(path = "filters_join.html")]
struct VecJoinTemplate {
    s: Vec<String>,
}

#[test]
fn test_vec_join() {
    let t = VecJoinTemplate {
        s: vec!["foo".into(), "bar".into(), "bazz".into()],
    };
    assert_eq!(t.render().unwrap(), "foo, bar, bazz");
}

#[derive(Template)]
#[template(path = "json.html")]
struct JsonTemplate<'a> {
    foo: &'a str,
    bar: &'a Value,
}

#[test]
fn test_json() {
    let val = json!({"arr": [ "one", 2, true, null ]});
    let t = JsonTemplate {
        foo: "a",
        bar: &val,
    };
    // Note: the json filter lacks a way to specify initial indentation
    assert_eq!(
        t.render().unwrap(),
        r#"{
  "foo": "a",
  "bar": {
  "arr": [
    "one",
    2,
    true,
    null
  ]
}
}"#
    );
}

#[derive(Template)]
#[template(source = "{{ x|mytrim|safe }}", ext = "html")]
struct NestedFilterTemplate {
    x: String,
}

#[test]
fn test_nested_filter_ref() {
    let t = NestedFilterTemplate {
        x: " floo & bar".to_string(),
    };
    assert_eq!(t.render().unwrap(), "floo & bar");
}

#[derive(Template)]
#[template(source = "{% let p = baz.print(foo.as_ref()) %}{{ p|upper }}", ext = "html")]
struct FilterLetFilterTemplate {
    foo: String,
    baz: Baz,
}

struct Baz {}

impl Baz {
    fn print(&self, s: &str) -> String {
        s.trim().to_owned()
    }
}

#[test]
fn test_filter_let_filter() {
    let t = FilterLetFilterTemplate {
        foo: " bar ".to_owned(),
        baz: Baz {},
    };
    assert_eq!(t.render().unwrap(), "BAR");
}
