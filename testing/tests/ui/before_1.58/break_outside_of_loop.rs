use askama::Template;

#[derive(Template)]
#[template(
    source = "Have a {%break%}, have a parsing error!",
    ext = "txt"
)]
struct MyTemplate;

fn main() {
}
