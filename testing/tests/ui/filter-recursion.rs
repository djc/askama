use askama::Template;

#[derive(Template)]
#[template(path = "filter-recursion.html")]
struct Filtered {
    s: &'static str,
}

fn main() {}
