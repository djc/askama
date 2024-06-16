#[cfg(feature = "serde-json")]
#[macro_use]
extern crate serde_json;

use askama::Template;
#[cfg(feature = "serde-json")]
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
        "// my &lt;html&gt; is &quot;unsafe&quot; &amp; \
         should be &#x27;escaped&#x27;"
    );
}

#[derive(Template)]
#[template(
    source = "{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\"|escape(\"none\") }}
{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\"|escape(\"html\") }}
{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\"|escape }}
{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\" }}
",
    ext = "txt",
    escape = "none"
)]
struct OptEscaperNoneTemplate;

#[test]
fn filter_opt_escaper_none() {
    let t = OptEscaperNoneTemplate;
    assert_eq!(
        t.render().unwrap(),
        r#"<h1 class="title">Foo Bar</h1>
&lt;h1 class=&quot;title&quot;&gt;Foo Bar&lt;/h1&gt;
<h1 class="title">Foo Bar</h1>
<h1 class="title">Foo Bar</h1>
"#
    );
}

#[derive(Template)]
#[template(
    source = "{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\"|escape(\"none\") }}
{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\"|escape(\"html\") }}
{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\"|escape }}
{{ \"<h1 class=\\\"title\\\">Foo Bar</h1>\" }}
",
    ext = "txt",
    escape = "html"
)]
struct OptEscaperHtmlTemplate;

#[test]
fn filter_opt_escaper_html() {
    let t = OptEscaperHtmlTemplate;
    assert_eq!(
        t.render().unwrap(),
        r#"<h1 class="title">Foo Bar</h1>
&lt;h1 class=&quot;title&quot;&gt;Foo Bar&lt;/h1&gt;
&lt;h1 class=&quot;title&quot;&gt;Foo Bar&lt;/h1&gt;
&lt;h1 class=&quot;title&quot;&gt;Foo Bar&lt;/h1&gt;
"#
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
#[template(source = "{{ var|fmt(\"{:?}\") }}", ext = "html", escape = "none")]
struct FmtTemplate<'a> {
    var: &'a str,
}

#[test]
fn filter_fmt() {
    let t = FmtTemplate { var: "formatted" };
    assert_eq!(t.render().unwrap(), "\"formatted\"");
}

#[derive(Template)]
#[template(
    source = "{{ 1|into_f64 }} {{ 1.9|into_isize }}",
    ext = "txt",
    escape = "none"
)]
struct IntoNumbersTemplate;

#[test]
fn into_numbers_fmt() {
    let t = IntoNumbersTemplate;
    assert_eq!(t.render().unwrap(), "1 1");
}

#[derive(Template)]
#[template(source = "{{ s|myfilter }}", ext = "txt")]
struct MyFilterTemplate<'a> {
    s: &'a str,
}

mod filters {
    pub fn myfilter(s: &str) -> ::askama::Result<String> {
        Ok(s.replace("oo", "aa"))
    }
    // for test_nested_filter_ref
    pub fn mytrim(s: &dyn (::std::fmt::Display)) -> ::askama::Result<String> {
        Ok(s.to_string().trim().to_owned())
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

#[cfg(feature = "serde-json")]
#[derive(Template)]
#[template(
    source = r#"{
  "foo": "{{ foo }}",
  "bar": {{ bar|json|safe }}
}"#,
    ext = "txt"
)]
struct JsonTemplate<'a> {
    foo: &'a str,
    bar: &'a Value,
}

#[cfg(feature = "serde-json")]
#[test]
fn test_json() {
    let val = json!({"arr": [ "one", 2, true, null ]});
    let t = JsonTemplate {
        foo: "a",
        bar: &val,
    };
    assert_eq!(
        t.render().unwrap(),
        r#"{
  "foo": "a",
  "bar": {"arr":["one",2,true,null]}
}"#
    );
}

#[cfg(feature = "serde-json")]
#[derive(Template)]
#[template(
    source = r#"{
  "foo": "{{ foo }}",
  "bar": {{ bar|json(2)|safe }}
}"#,
    ext = "txt"
)]
struct PrettyJsonTemplate<'a> {
    foo: &'a str,
    bar: &'a Value,
}

#[cfg(feature = "serde-json")]
#[test]
fn test_pretty_json() {
    let val = json!({"arr": [ "one", 2, true, null ]});
    let t = PrettyJsonTemplate {
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

#[cfg(feature = "serde-json")]
#[derive(Template)]
#[template(source = r#"{{ bar|json(indent)|safe }}"#, ext = "txt")]
struct DynamicJsonTemplate<'a> {
    bar: &'a Value,
    indent: Option<&'a str>,
}

#[cfg(feature = "serde-json")]
#[test]
fn test_dynamic_json() {
    let val = json!({"arr": ["one", 2]});
    let t = DynamicJsonTemplate {
        bar: &val,
        indent: Some("?"),
    };
    assert_eq!(
        t.render().unwrap(),
        r#"{
?"arr": [
??"one",
??2
?]
}"#
    );

    let t = DynamicJsonTemplate {
        bar: &val,
        indent: None,
    };
    assert_eq!(t.render().unwrap(), r#"{"arr":["one",2]}"#);
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
#[template(
    source = "{% let p = baz.print(foo.as_ref()) %}{{ p|upper }}",
    ext = "html"
)]
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

#[derive(Template)]
#[template(source = "{{ foo|truncate(10) }}{{ foo|truncate(5) }}", ext = "txt")]
struct TruncateFilter {
    foo: String,
}

#[test]
fn test_filter_truncate() {
    let t = TruncateFilter {
        foo: "alpha bar".into(),
    };
    assert_eq!(t.render().unwrap(), "alpha baralpha...");
}

#[cfg(feature = "serde-json")]
#[derive(Template)]
#[template(source = r#"<li data-name="{{name|json}}"></li>"#, ext = "html")]
struct JsonAttributeTemplate<'a> {
    name: &'a str,
}

#[cfg(feature = "serde-json")]
#[test]
fn test_json_attribute() {
    let t = JsonAttributeTemplate {
        name: r#""><button>Hacked!</button>"#,
    };
    assert_eq!(
        t.render().unwrap(),
        r#"<li data-name="&quot;\&quot;\u003e\u003cbutton\u003eHacked!\u003c/button\u003e&quot;"></li>"#
    );
}

#[cfg(feature = "serde-json")]
#[derive(Template)]
#[template(source = r#"<li data-name='{{name|json|safe}}'></li>"#, ext = "html")]
struct JsonAttribute2Template<'a> {
    name: &'a str,
}

#[cfg(feature = "serde-json")]
#[test]
fn test_json_attribute2() {
    let t = JsonAttribute2Template {
        name: r#"'><button>Hacked!</button>"#,
    };
    assert_eq!(
        t.render().unwrap(),
        r#"<li data-name='"\u0027\u003e\u003cbutton\u003eHacked!\u003c/button\u003e"'></li>"#
    );
}

#[cfg(feature = "serde-json")]
#[derive(Template)]
#[template(
    source = r#"<script>var user = {{name|json|safe}}</script>"#,
    ext = "html"
)]
struct JsonScriptTemplate<'a> {
    name: &'a str,
}

#[cfg(feature = "serde-json")]
#[test]
fn test_json_script() {
    let t = JsonScriptTemplate {
        name: r#"</script><button>Hacked!</button>"#,
    };
    assert_eq!(
        t.render().unwrap(),
        r#"<script>var user = "\u003c/script\u003e\u003cbutton\u003eHacked!\u003c/button\u003e"</script>"#
    );
}

#[derive(askama::Template)]
#[template(
    source = r#"{% let word = s|as_ref %}{{ word }}
{%- let hello = String::from("hello") %}
{%- if word|deref == hello %}1{% else %}2{% endif %}"#,
    ext = "html"
)]
struct LetBorrow {
    s: String,
}

#[test]
fn test_let_borrow() {
    let template = LetBorrow {
        s: "hello".to_owned(),
    };
    assert_eq!(template.render().unwrap(), "hello1")
}
