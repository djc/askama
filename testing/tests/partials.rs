use askama::Template;

// Partials let us render a portion of a larger template without needing
// to extract those portions into their own files (and then using `{%
// include ... %}` to bring them back into the parent template). The
// normal use of a partial is to first render the entire template, then
// render partials as the application data changes, sending just those
// partial renderings to the client in order to update targeted portions
// of the UI.
//
// First, we show that a template containing block definitions can be
// rendered just like normal:
#[derive(Template)]
#[template(path = "partials.html")]
struct AllSectionTemplate<'a> {
    first: &'a str,
    second_a: &'a str,
    second_b: &'a str,
    third: &'a str,
}

#[test]
fn test_all_sections() {
    let t = AllSectionTemplate {
        first: "1",
        second_a: "2A",
        second_b: "2B",
        third: "3",
    };
    assert_eq!(
        t.render().unwrap(),
        "<div>
    <div>First section: 1</div>

    <div>
        <span>Second section, subsection A: 2A</span>
        <span>Second section, subsection B: 2B</span>
    </div>

    <div>Third section: 3</div>
</div>"
    );
}

// Then we can create a template that focuses on a specific block and
// render just that partial content (note that we only need to include
// the fields in the template struct that are actually used by this
// partial):
#[derive(Template)]
#[template(path = "partials.html", block = "first")]
struct FirstPartial<'a> {
    first: &'a str,
}

#[test]
fn test_first_partial() {
    let t = FirstPartial { first: "One" };
    assert_eq!(
        t.render().unwrap(),
        "
    <div>First section: One</div>"
    );
}

// Partials can target a block with nested blocks:
#[derive(Template)]
#[template(path = "partials.html", block = "second")]
struct SecondPartial<'a> {
    second_a: &'a str,
    second_b: &'a str,
}

#[test]
fn test_second_partial() {
    let t = SecondPartial {
        second_a: "Two A",
        second_b: "Or not two A?",
    };
    assert_eq!(
        t.render().unwrap(),
        "
    <div>
        <span>Second section, subsection A: Two A</span>
        <span>Second section, subsection B: Or not two A?</span>
    </div>"
    );
}

// ...or partials can target a deeply-nested block:
#[derive(Template)]
#[template(path = "partials.html", block = "second_b")]
struct SecondBPartial<'a> {
    second_b: &'a str,
}

#[test]
fn test_second_b_partial() {
    let t = SecondBPartial { second_b: "Two B" };
    assert_eq!(
        t.render().unwrap(),
        "
        <span>Second section, subsection B: Two B</span>"
    );
}

// Note that while you *can* target a block in a child template, this is
// the less common way to use a partial *and* requires that the child
// specifically reference the block. So this works, because `child.html`
// overrides the `content` block:
#[derive(Template)]
#[template(path = "child.html", block = "content")]
struct ChildContentPartial<'a> {
    title: &'a str,
}

#[test]
fn test_child_content_partial() {
    let t = ChildContentPartial { title: "Bar" };
    assert_eq!(t.render().unwrap(), "(Bar) Content goes here");
}

// ...but this does *not* work, because the `foo` block is only
// mentioned in the `base.html` template and so cannot be targeted by
// the `block` attribute that locates a partial:
//
// #[derive(Template)]
// #[template(path = "child.html", block = "foo")]
// struct ChildInheritsFooPartial<'a> {
//     title: &'a str,
// }
//
// #[test]
// fn test_child_inherits_foo_partial() {
//     let t = ChildFooPartial { title: "Bar" };
//     assert_eq!(t.render().unwrap(), "Foo");
// }

// This test, however, does work, because the child explicitly names the
// `foo` block and so we can refer to it in a partial.
#[derive(Template)]
#[template(
    source = "{% extends \"base.html\" %}{% block foo %}FooChild{% endblock %}",
    ext = "html",
    block = "foo"
)]
struct ChildSpecifiesFooPartial {}

#[test]
fn test_child_specifies_foo_partial() {
    let t = ChildSpecifiesFooPartial {};
    assert_eq!(t.render().unwrap(), "FooChild");
}
