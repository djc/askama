use askama::Template;

#[derive(Template)]
#[template(
    source = r#"{%- extends "whatever.html" %}"#,
    ext = "html"
)]
struct ExtendsPre;

#[derive(Template)]
#[template(
    source = r#"{% extends "whatever.html" -%}"#,
    ext = "html"
)]
struct ExtendsPost;

fn main() {
}
