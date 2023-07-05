use askama::Template;

macro_rules! hello {
    () => {
        "world"
    };
}

#[derive(Template)]
#[template(path = "rust-macros.html")]
struct RustMacrosTemplate {}

#[test]
fn main() {
    let template = RustMacrosTemplate {};
    assert_eq!("Hello, world!", template.render().unwrap());
}

mod foo {
    macro_rules! hello {
        () => {
            "world"
        };
    }

    pub(crate) use hello;
}

#[derive(Template)]
#[template(path = "rust-macros-full-path.html")]
struct RustMacrosFullPathTemplate {}

#[test]
fn full_path() {
    let template = RustMacrosFullPathTemplate {};
    assert_eq!("Hello, world!", template.render().unwrap());
}

macro_rules! call_a_or_b_on_tail {
    ((a: $a:expr, b: $b:expr, c: $c:expr), call a: $($tail:expr),*) => {
        ($a)($($tail),*)
    };

    ((a: $a:expr, b: $b:expr, c: $c:expr), call b: $($tail:expr),*) => {
        ($b)($($tail),*)
    };

    ((a: $a:expr, b: $b:expr, c: $c:expr), call c: $($tail:expr),*) => {
        ($c)($($tail),*)
    };
}

fn year(y: u16, _: &str, _: u8) -> u16 {
    y
}

fn month(_: u16, m: &str, _: u8) -> &str {
    m
}

fn day(_: u16, _: &str, d: u8) -> u8 {
    d
}

#[derive(Template)]
#[template(path = "rust-macro-args.html")]
struct RustMacrosArgTemplate {}

#[test]
fn args() {
    let template = RustMacrosArgTemplate {};
    assert_eq!("2021\nJuly\n2", template.render().unwrap());
}
