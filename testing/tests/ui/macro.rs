use askama::Template;

#[derive(Template)]
#[template(source = "{%- macro thrice(param) -%}
{{ param }}
{%- endmacro -%}

{%- call thrice(2, 3) -%}", ext = "html")]
struct InvalidNumberOfArgs;

#[derive(Template)]
#[template(source = "{%- macro thrice(param, param2) -%}
{{ param }} {{ param2 }}
{%- endmacro -%}

{%- call thrice() -%}", ext = "html")]
struct InvalidNumberOfArgs2;

#[derive(Template)]
#[template(source = "{%- macro thrice() -%}
{%- endmacro -%}

{%- call thrice(1, 2) -%}", ext = "html")]
struct InvalidNumberOfArgs3;

fn main() {
}
