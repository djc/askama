use askama::Template;

#[derive(Template)]
#[template(path = "fragment-simple.html", block = "body")]
struct FragmentSimple<'a> {
    name: &'a str,
}

/// Tests a simple base-inherited template with block fragment rendering.
#[test]
fn test_fragment_simple() {
    let simple = FragmentSimple { name: "world" };

    assert_eq!(simple.render().unwrap(), "\n<p>Hello world!</p>\n");
}

#[derive(Template)]
#[template(path = "fragment-super.html", block = "body")]
struct FragmentSuper<'a> {
    name: &'a str,
}

/// Tests a case where a block fragment rendering calls the parent.
/// Single inheritance only.
#[test]
fn test_fragment_super() {
    let sup = FragmentSuper { name: "world" };

    assert_eq!(
        sup.render().unwrap(),
        "\n<p>Hello world!</p>\n\n<p>Parent body content</p>\n\n"
    );
}

#[derive(Template)]
#[template(path = "fragment-nested-block.html", block = "nested")]
struct FragmentNestedBlock;

/// Tests rendering a block fragment inside of a block.
#[test]
fn test_fragment_nested_block() {
    let nested_block = FragmentNestedBlock {};

    assert_eq!(
        nested_block.render().unwrap(),
        "\n<p>I should be here.</p>\n"
    );
}

#[derive(Template)]
#[template(path = "fragment-nested-super.html", block = "body")]
struct FragmentNestedSuper<'a> {
    name: &'a str,
}

/// Tests rendering a block fragment with multiple inheritance.
/// The middle parent adds square brackets around the base.
#[test]
fn test_fragment_nested_super() {
    let nested_sup = FragmentNestedSuper { name: "world" };

    assert_eq!(
        nested_sup.render().unwrap(),
        "\n<p>Hello world!</p>\n\n[\n<p>Parent body content</p>\n]\n\n"
    );
}

#[derive(Template)]
#[template(path = "fragment-unused-expr.html", block = "body")]
struct FragmentUnusedExpr<'a> {
    required: &'a str,
}

/// Tests a case where an expression is defined outside of a block fragment
/// Ideally, the struct isn't required to define that field.
#[test]
fn test_fragment_unused_expression() {
    let unused_expr = FragmentUnusedExpr {
        required: "Required",
    };

    assert_eq!(unused_expr.render().unwrap(), "\n<p>Required</p>\n");
}

#[derive(Template)]
#[template(path = "blocks.txt", block = "index")]
struct RenderInPlace<'a> {
    s1: Section<'a>,
}

#[derive(Template)]
#[template(path = "blocks.txt", block = "section")]
struct Section<'a> {
    values: &'a [&'a str],
}

#[test]
fn test_specific_block() {
    let s1 = Section {
        values: &["a", "b", "c"],
    };
    assert_eq!(s1.render().unwrap(), "[abc]");
    let t = RenderInPlace { s1 };
    assert_eq!(t.render().unwrap(), "\nSection: [abc]\n");
}

#[derive(Template)]
#[template(
    source = r#"{% block empty %}
{% endblock %}

{% if let Some(var) = var %}
{{ var }}
{% endif %}"#,
    block = "empty",
    ext = "txt"
)]
struct Empty {}

#[test]
fn test_render_only_block() {
    assert_eq!(Empty {}.render().unwrap(), "\n");
}
