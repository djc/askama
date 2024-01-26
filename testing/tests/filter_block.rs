use askama::Template;

#[derive(Template)]
#[template(
    source = r#"{% filter lower %}
    {{ t }} / HELLO / {{ u }}
{% endfilter %}

{{ u|lower }}
"#,
    ext = "html"
)]
struct A<T, U = u8>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    t: T,
    u: U,
}

#[test]
fn filter_block_basic() {
    let template = A { t: "a", u: "B" };
    assert_eq!(template.render().unwrap(), "\n    a / hello / b\n\n\nb\n")
}

// This test ensures that we don't have variable shadowing when we have more than one
// filter block at the same location.
#[derive(Template)]
#[template(
    source = r#"{% filter lower %}
    {{ t }} / HELLO / {{ u }}
{% endfilter %}

{% filter upper %}
{{ u }} + TaDaM + {{ t }}
{% endfilter %}

{% filter lower %}
    {{ t }} - CHECK - {{ t }}
{% endfilter %}

{{ u|upper }}"#,
    ext = "html"
)]
struct B<T, U = u8>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    t: T,
    u: U,
}

#[test]
fn filter_block_shadowing() {
    let template = B { t: "a", u: "B" };
    assert_eq!(
        template.render().unwrap(),
        r#"
    a / hello / b



B + TADAM + A



    a - check - a


B"#
    );
}

// This test ensures that whitespace control is correctly handled.
#[derive(Template)]
#[template(
    source = r#"{% filter lower -%}
    {{ t }} / HELLO / {{ u }}
{% endfilter %}

{%- filter upper -%}
{{ u }} + TaDaM + {{ t }}
{%- endfilter -%}

++b"#,
    ext = "html"
)]
struct C<T, U = u8>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    t: T,
    u: U,
}

#[test]
fn filter_block_whitespace_control() {
    let template = C { t: "a", u: "B" };
    assert_eq!(
        template.render().unwrap(),
        r#"a / hello / b
B + TADAM + A++b"#
    );
}

// This test ensures that HTML escape is correctly handled.
#[derive(Template)]
#[template(source = r#"{% filter lower %}<block>{% endfilter %}"#, ext = "html")]
struct D;

#[test]
fn filter_block_html_escape() {
    let template = D;
    assert_eq!(template.render().unwrap(), r#"&lt;block&gt;"#);
}

// This test ensures that it is not escaped if it is not HTML.
#[derive(Template)]
#[template(source = r#"{% filter lower %}<block>{% endfilter %}"#, ext = "txt")]
struct E;

#[test]
fn filter_block_not_html_escape() {
    let template = E;
    assert_eq!(template.render().unwrap(), r#"<block>"#);
}
