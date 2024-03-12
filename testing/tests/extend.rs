use askama::Template;

#[derive(Template)]
#[template(
    source = r#"{% extends "extend_and_import.html" %}
{%- import "macro.html" as m2 -%}

{%- macro another(param) -%}

--> {{ param }}

{%- endmacro -%}

{% block header -%}
{% call m1::twice(1) %}
{% call m2::twice(2) %}
{% call another(3) %}
{%- endblock -%}
"#,
    ext = "txt"
)]
struct A;

#[test]
fn test_macro_in_block_inheritance() {
    assert_eq!(A.render().unwrap(), "\n\n1 1\n2 2\n--> 3");
}
