// Nb. this test fails because currently an empty array "[]" is always a syntax error in askama,
// but even if this changes, this test should keep failing, but possibly with another error message

use askama::Template;

#[derive(Template)]
#[template(
    source = r#"{% for v in values %}{{ loop.cycle([]) }}{{ v }},{% endfor %}"#,
    ext = "txt"
)]
struct ForCycleEmpty;

fn main() {
}
