#![allow(clippy::blacklisted_name)]

use askama::{SizedTemplate, Template};

use std::collections::HashMap;

#[derive(Template)]
#[template(path = "simple.html")]
struct VariablesTemplate<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables() {
    let s = VariablesTemplate {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(
        s.render().unwrap(),
        "\nhello world, foo\n\
         with number: 42\n\
         Iñtërnâtiônàlizætiøn is important\n\
         in vars too: Iñtërnâtiônàlizætiøn"
    );
    assert_eq!(
        <VariablesTemplate as SizedTemplate>::extension(),
        Some("html")
    );
}

#[derive(Template)]
#[template(path = "hello.html")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"'/" };

    assert_eq!(
        s.render().unwrap(),
        "Hello, &lt;&gt;&amp;&quot;&#x27;&#x2f;!"
    );
}

#[derive(Template)]
#[template(path = "simple-no-escape.txt")]
struct VariablesTemplateNoEscape<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables_no_escape() {
    let s = VariablesTemplateNoEscape {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(
        s.render().unwrap(),
        "\nhello world, foo\n\
         with number: 42\n\
         Iñtërnâtiônàlizætiøn is important\n\
         in vars too: Iñtërnâtiônàlizætiøn"
    );
}

#[derive(Template)]
#[template(path = "if.html")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let s = IfTemplate { cond: true };
    assert_eq!(s.render().unwrap(), "true");
}

#[derive(Template)]
#[template(path = "else.html")]
struct ElseTemplate {
    cond: bool,
}

#[test]
fn test_else() {
    let s = ElseTemplate { cond: false };
    assert_eq!(s.render().unwrap(), "false");
}

#[derive(Template)]
#[template(path = "else-if.html")]
struct ElseIfTemplate {
    cond: bool,
    check: bool,
}

#[test]
fn test_else_if() {
    let s = ElseIfTemplate {
        cond: false,
        check: true,
    };
    assert_eq!(s.render().unwrap(), "checked");
}

#[derive(Template)]
#[template(path = "literals.html")]
struct LiteralsTemplate {}

#[test]
fn test_literals() {
    let s = LiteralsTemplate {};
    assert_eq!(s.render().unwrap(), "a\na\ntrue\nfalse");
}

#[derive(Template)]
#[template(path = "literals-escape.html")]
struct LiteralsEscapeTemplate {}

#[test]
fn test_literals_escape() {
    let s = LiteralsEscapeTemplate {};
    assert_eq!(
        s.render().unwrap(),
        "A\n\r\t\\\0♥&#x27;&quot;&quot;\nA\n\r\t\\\0♥&#x27;&quot;&#x27;"
    );
}

struct Holder {
    a: usize,
}

#[derive(Template)]
#[template(path = "attr.html")]
struct AttrTemplate {
    inner: Holder,
}

#[test]
fn test_attr() {
    let t = AttrTemplate {
        inner: Holder { a: 5 },
    };
    assert_eq!(t.render().unwrap(), "5");
}

#[derive(Template)]
#[template(path = "tuple-attr.html")]
struct TupleAttrTemplate<'a> {
    tuple: (&'a str, &'a str),
}

#[test]
fn test_tuple_attr() {
    let t = TupleAttrTemplate {
        tuple: ("foo", "bar"),
    };
    assert_eq!(t.render().unwrap(), "foobar");
}

struct NestedHolder {
    holder: Holder,
}

#[derive(Template)]
#[template(path = "nested-attr.html")]
struct NestedAttrTemplate {
    inner: NestedHolder,
}

#[test]
fn test_nested_attr() {
    let t = NestedAttrTemplate {
        inner: NestedHolder {
            holder: Holder { a: 5 },
        },
    };
    assert_eq!(t.render().unwrap(), "5");
}

#[derive(Template)]
#[template(path = "option.html")]
struct OptionTemplate<'a> {
    var: Option<&'a str>,
}

#[test]
fn test_option() {
    let some = OptionTemplate { var: Some("foo") };
    assert_eq!(some.render().unwrap(), "some: foo");
    let none = OptionTemplate { var: None };
    assert_eq!(none.render().unwrap(), "none");
}

#[derive(Template)]
#[template(path = "generics.html")]
struct GenericsTemplate<T, U = u8>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    t: T,
    u: U,
}

#[test]
fn test_generics() {
    let t = GenericsTemplate { t: "a", u: 42 };
    assert_eq!(t.render().unwrap(), "a42");
}

#[derive(Template)]
#[template(path = "composition.html")]
struct CompositionTemplate {
    foo: IfTemplate,
}

#[test]
fn test_composition() {
    let t = CompositionTemplate {
        foo: IfTemplate { cond: true },
    };
    assert_eq!(t.render().unwrap(), "composed: true");
}

#[derive(PartialEq, Eq)]
enum Alphabet {
    Alpha,
}

#[derive(Template)]
#[template(source = "{% if x == Alphabet::Alpha %}true{% endif %}", ext = "txt")]
struct PathCompareTemplate {
    x: Alphabet,
}

#[test]
fn test_path_compare() {
    let t = PathCompareTemplate { x: Alphabet::Alpha };
    assert_eq!(t.render().unwrap(), "true");
}

#[derive(Template)]
#[template(
    source = "{% for i in [\"a\", \"\"] %}{{ i }}{% endfor %}",
    ext = "txt"
)]
struct ArrayTemplate {}

#[test]
fn test_slice_literal() {
    let t = ArrayTemplate {};
    assert_eq!(t.render().unwrap(), "a");
}

#[derive(Template)]
#[template(source = "Hello, {{ world(\"123\", 4) }}!", ext = "txt")]
struct FunctionRefTemplate {
    world: fn(s: &str, v: &u8) -> String,
}

#[test]
fn test_func_ref_call() {
    let t = FunctionRefTemplate {
        world: |s, r| format!("world({}, {})", s, r),
    };
    assert_eq!(t.render().unwrap(), "Hello, world(123, 4)!");
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn world2(s: &str, v: &u8) -> String {
    format!("world{}{}", v, s)
}

#[derive(Template)]
#[template(source = "Hello, {{ self::world2(\"123\", 4) }}!", ext = "txt")]
struct PathFunctionTemplate;

#[test]
fn test_path_func_call() {
    assert_eq!(PathFunctionTemplate.render().unwrap(), "Hello, world4123!");
}

#[derive(Template)]
#[template(source = "{{ ::std::string::ToString::to_string(123) }}", ext = "txt")]
struct RootPathFunctionTemplate;

#[test]
fn test_root_path_func_call() {
    assert_eq!(RootPathFunctionTemplate.render().unwrap(), "123");
}

#[derive(Template)]
#[template(source = "Hello, {{ Self::world3(self, \"123\", 4) }}!", ext = "txt")]
struct FunctionTemplate;

impl FunctionTemplate {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn world3(&self, s: &str, v: &u8) -> String {
        format!("world{}{}", s, v)
    }
}

#[derive(Template)]
#[template(source = "  {# foo -#} ", ext = "txt")]
struct CommentTemplate {}

#[test]
fn test_comment() {
    let t = CommentTemplate {};
    assert_eq!(t.render().unwrap(), "  ");
}

#[derive(Template)]
#[template(source = "{% if !foo %}Hello{% endif %}", ext = "txt")]
struct NegationTemplate {
    foo: bool,
}

#[test]
fn test_negation() {
    let t = NegationTemplate { foo: false };
    assert_eq!(t.render().unwrap(), "Hello");
}

#[derive(Template)]
#[template(source = "{% if foo > -2 %}Hello{% endif %}", ext = "txt")]
struct MinusTemplate {
    foo: i8,
}

#[test]
fn test_minus() {
    let t = MinusTemplate { foo: 1 };
    assert_eq!(t.render().unwrap(), "Hello");
}

#[derive(Template)]
#[template(source = "{{ foo[\"bar\"] }}", ext = "txt")]
struct IndexTemplate {
    foo: HashMap<String, String>,
}

#[test]
fn test_index() {
    let mut foo = HashMap::new();
    foo.insert("bar".into(), "baz".into());
    let t = IndexTemplate { foo };
    assert_eq!(t.render().unwrap(), "baz");
}

#[derive(Template)]
#[template(source = "foo", ext = "txt")]
struct Empty;

#[test]
fn test_empty() {
    assert_eq!(Empty.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "raw-simple.html")]
struct RawTemplate;

#[test]
fn test_raw_simple() {
    let template = RawTemplate;
    assert_eq!(template.render().unwrap(), "\n<span>{{ name }}</span>\n");
}

#[derive(Template)]
#[template(path = "raw-complex.html")]
struct RawTemplateComplex;

#[test]
fn test_raw_complex() {
    let template = RawTemplateComplex;
    assert_eq!(
        template.render().unwrap(),
        "\n{% block name %}\n  <span>{{ name }}</span>\n{% endblock %}\n"
    );
}

mod without_import_on_derive {
    #[derive(askama::Template)]
    #[template(source = "foo", ext = "txt")]
    struct WithoutImport;

    #[test]
    fn test_without_import() {
        use askama::Template;
        assert_eq!(WithoutImport.render().unwrap(), "foo");
    }
}

#[derive(askama::Template)]
#[template(source = "{% let s = String::new() %}{{ s }}", ext = "txt")]
struct DefineStringVar;

#[test]
fn test_define_string_var() {
    let template = DefineStringVar;
    assert_eq!(template.render().unwrap(), "");
}
