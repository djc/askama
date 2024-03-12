use askama::Template;

#[derive(Template)]
#[template(source = r#"{% extends "extend_and_import.html" %}

{% let x = 12 %}
{% block header -%}
{{ x }}
{% endblock %}"#, ext = "html")]
struct A;

fn main() {
}
