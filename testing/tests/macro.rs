use askama::Template;

#[derive(Template)]
#[template(path = "macro.html")]
struct MacroTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_macro() {
    let t = MacroTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "12foo foo foo3");
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
