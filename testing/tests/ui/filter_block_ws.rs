use askama::Template;

#[derive(Template)]
#[template(source = "{% filter lower|indent(2) - %}
HELLO
{{v}}
{%- endfilter %}", ext = "html")]
struct A;

fn main() {
    A.render().unwrap();
}
