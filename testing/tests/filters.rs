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
#[template(source = "{{ s|myfilter }}", ext = "txt")]
struct MyFilterTemplate<'a> {
    s: &'a str,
}

mod filters {
    pub fn myfilter(s: &str) -> ::askama::Result<String> {
        Ok(s.replace("oo", "aa").to_string())
    }
}

#[test]
fn test_my_filter() {
    let t = MyFilterTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "faa");
}


#[derive(Template)]
#[template(path= "filters_join.html")]
struct JoinTemplate<'a> {
    s: &'a [&'a str],
}

#[test]
fn test_join() {
    let t = JoinTemplate { s: &["foo", "bar", "bazz"] };
    assert_eq!(t.render().unwrap(), "foo, bar, bazz");
}

#[derive(Template)]
#[template(path= "filters_join.html")]
struct VecJoinTemplate {
    s: Vec<String>
}

#[test]
fn test_vec_join() {
    let t = VecJoinTemplate { s: vec!["foo".into(), "bar".into(), "bazz".into()] };
    assert_eq!(t.render().unwrap(), "foo, bar, bazz");
}
