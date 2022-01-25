use askama::Template;

#[derive(Template)]
#[template(ext = "txt", "unnamed")]
struct UnnamedTemplateArgument;

fn main() {
}
