use askama::Template;

#[derive(Template)]
#[template(source = "{{ s|a|a|a|a|a|a|a|A|a|a|a|a|a|a|a|a|a|a|a|a|a", ext = "txt")]
struct Filtered {
    s: &'static str,
}

fn main() {}
