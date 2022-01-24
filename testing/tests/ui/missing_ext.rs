use askama::Template;

#[derive(Template)]
#[template(source = "🙂")]
struct MissingExt;

fn main() {
}
