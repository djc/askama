use askama::Template;

#[derive(Template)]
#[template(
    source = r#"{% for v in values %}{{ loop.cycle("r", "g", "b") }}{{ v }},{% endfor %}"#,
    ext = "txt"
)]
struct ForCycle<'a> {
    values: &'a [u8],
}

fn main() {
}
