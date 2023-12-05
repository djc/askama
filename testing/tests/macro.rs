use askama::Template;

#[derive(Template)]
#[template(path = "macro.html")]
struct MacroTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_macro() {
    let t = MacroTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "12foo foo foo34foo foo5");
}

#[derive(Template)]
#[template(path = "macro-no-args.html")]
struct MacroNoArgsTemplate;

#[test]
fn test_macro_no_args() {
    let t = MacroNoArgsTemplate;
    assert_eq!(t.render().unwrap(), "11the best thing111we've ever done11");
}

#[derive(Template)]
#[template(path = "import.html")]
struct ImportTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_import() {
    let t = ImportTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo foo foo");
}

#[derive(Template)]
#[template(path = "deep-nested-macro.html")]
struct NestedTemplate;

#[test]
fn test_nested() {
    let t = NestedTemplate;
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "deep-import-parent.html")]
struct DeepImportTemplate;

#[test]
fn test_deep_import() {
    let t = DeepImportTemplate;
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "macro-short-circuit.html")]
struct ShortCircuitTemplate {}

#[test]
fn test_short_circuit() {
    let t = ShortCircuitTemplate {};
    assert_eq!(t.render().unwrap(), "truetruetruefalsetruetrue");
}

#[derive(Template)]
#[template(path = "nested-macro-args.html")]
struct NestedMacroArgsTemplate {}

#[test]
fn test_nested_macro_with_args() {
    let t = NestedMacroArgsTemplate {};
    assert_eq!(t.render().unwrap(), "first second");
}

#[derive(Template)]
#[template(path = "macro-import-str-cmp.html")]
struct StrCmpTemplate;

#[test]
fn str_cmp() {
    let t = StrCmpTemplate;
    assert_eq!(t.render().unwrap(), "AfooBotherCneitherD");
}

#[derive(Template)]
#[template(path = "macro-self-arg.html")]
struct MacroSelfArgTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_macro_self_arg() {
    let t = MacroSelfArgTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(
    source = "{%- macro thrice(param1, param2) -%}
{{ param1 }} {{ param2 }}
{% endmacro -%}

{%- call thrice(param1=2, param2=3) -%}
{%- call thrice(param2=3, param1=2) -%}
{%- call thrice(3, param2=2) -%}
",
    ext = "html"
)]
struct MacroNamedArg;

#[test]
// We check that it's always the correct values passed to the
// expected argument.
fn test_named_argument() {
    assert_eq!(
        MacroNamedArg.render().unwrap(),
        "\
2 3
2 3
3 2
"
    );
}

#[derive(Template)]
#[template(
    source = r#"{% macro button(label) %}
{{- label -}}
{% endmacro %}

{%- call button(label="hi") -%}
"#,
    ext = "html"
)]
struct OnlyNamedArgument;

#[test]
fn test_only_named_argument() {
    assert_eq!(OnlyNamedArgument.render().unwrap(), "hi");
}
