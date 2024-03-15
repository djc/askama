use askama::Template;

#[derive(Template)]
#[template(source = "{%- macro thrice(param1, param2) -%}
{{ param1 }} {{ param2 }}
{%- endmacro -%}

{%- call thrice(param1=2, param3=3) -%}", ext = "html")]
struct InvalidNamedArg;

#[derive(Template)]
#[template(source = "{%- macro thrice(param1, param2) -%}
{{ param1 }} {{ param2 }}
{%- endmacro -%}

{%- call thrice(param1=2, param1=3) -%}", ext = "html")]
struct InvalidNamedArg2;

// Ensures that filters can't have named arguments.
#[derive(Template)]
#[template(source = "{%- macro thrice(param1, param2) -%}
{{ param1 }} {{ param2 }}
{%- endmacro -%}

{%- call thrice(3, param1=2) | filter(param1=12) -%}", ext = "html")]
struct InvalidNamedArg3;

// Ensures that named arguments can only be passed last.
#[derive(Template)]
#[template(source = "{%- macro thrice(param1, param2) -%}
{{ param1 }} {{ param2 }}
{%- endmacro -%}
{%- call thrice(param1=2, 3) -%}", ext = "html")]
struct InvalidNamedArg4;

// Ensures that named arguments can't be used for arguments before them.
#[derive(Template)]
#[template(source = "{%- macro thrice(param1, param2) -%}
{{ param1 }} {{ param2 }}
{%- endmacro -%}
{%- call thrice(3, param1=2) -%}", ext = "html")]
struct InvalidNamedArg5;

fn main() {
}
