use askama::Template;

#[derive(Template)]
#[template(
    source = "{%for i in 1..=10%}{{i}}{%endfo%}\n1234567890123456789012345678901234567890",
    ext = "txt"
)]
struct MyTemplate;

fn main() {
}
