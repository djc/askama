use askama::Template;

#[derive(Template)]
#[template(no_such_argument = "fail")]
struct UnknownTemplateArgument;

fn main() {
}
