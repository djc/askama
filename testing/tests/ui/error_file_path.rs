use askama::Template;

#[derive(Template)]
#[template(path = "invalid_syntax.html")]
struct A;

#[derive(Template)]
#[template(path = "include_invalid_syntax.html")]
struct B;

#[derive(Template)]
#[template(source = r#"{% extends "include_invalid_syntax.html" %}"#, ext = "txt")]
struct C;

fn main() {
}
