#[macro_use]
extern crate askama;

use askama::Template;

macro_rules! hello {
    ($name:expr) => {
        "world"
    }
}

#[derive(Template)]
#[template(path = "rust-macros.html")]
struct RustMacrosTemplate<'a> {
    name: &'a str,
}

#[test]
fn main() {
    let template = RustMacrosTemplate { name: "foo" };
    assert_eq!("Hello, world!", template.render().unwrap());
}
