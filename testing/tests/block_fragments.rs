use askama::Template;

// fragment-nested-block.html
// fragment-nested-super.html
// fragment-simple.html
// fragment-super.html

#[derive(Template)]
#[template(path = "fragment-simple.html", block = "body")]
struct FragmentSimple<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "fragment-super.html", block = "body")]
struct FragmentSuper<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "fragment-nested-block.html", block = "nested")]
struct FragmentNestedBlock;

#[derive(Template)]
#[template(path = "fragment-nested-super.html", block = "body")]
struct FragmentNestedSuper<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "fragment-unused-expr.html", block = "body")]
struct FragmentUnusedExpr<'a> {
    required: &'a str,
}

#[test]
fn test_fragment_simple() {
    let simple = FragmentSimple { name: "world" };

    assert_eq!(
        simple.render().unwrap(),
        "\n\n<p>Hello world!</p>\n"
    );
}

#[test]
fn test_fragment_super() {
    let sup = FragmentSuper { name: "world" };

    assert_eq!(
        sup.render().unwrap(),
        "\n\n<p>Hello world!</p>\n\n<p>Parent body content</p>\n\n"
    );
}

#[test]
fn test_fragment_nested_block() {
    let nested_block = FragmentNestedBlock {};

    assert_eq!(
        nested_block.render().unwrap(),
        "\n\n<p>I should be here.</p>\n"
    );
}

#[test]
fn test_fragment_nested_super() {
    let nested_sup = FragmentNestedSuper { name: "world" };

    assert_eq!(
        nested_sup.render().unwrap(),
        "\n\n<p>Hello world!</p>\n\n[\n<p>Parent body content</p>\n]\n\n"
    );
}

#[test]
fn test_fragment_unused_expression() {
    let unused_expr = FragmentUnusedExpr { required: "Required" };

    assert_eq!(
        unused_expr.render().unwrap(),
        "\n\n<p>Required</p>\n"
    );
}
