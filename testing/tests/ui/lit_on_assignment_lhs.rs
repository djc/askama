use askama::Template;

#[derive(Template)]
#[template(
    source = "{%let 7=x%}",
    ext = "txt"
)]
struct MyTemplate;

fn main() {
}
