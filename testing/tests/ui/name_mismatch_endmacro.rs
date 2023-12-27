use askama::Template;

#[derive(Template)]
#[template(source = "{% macro foo(arg) %} {{arg}} {% endmacro not_foo %}", ext = "html")]
struct NameMismatchEndMacro;

fn main() {
}
