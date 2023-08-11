use askama::Template;

#[derive(Template)]
#[template(
    source = r#"{% for v in values %}{{ loop.cycle([]) }}{{ v }},{% endfor %}"#,
    ext = "txt"
)]
struct ForCycleEmpty;

fn main() {
}
