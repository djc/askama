use askama::Template;

#[derive(Template)]
#[template(source = "{% if true %}{% elif false %}{% endif %}", ext = "html")]
struct UnknownElif;

fn main() {
}
