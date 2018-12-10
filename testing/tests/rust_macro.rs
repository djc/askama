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

macro_rules! call_a_or_b_on_tail {
    ((a: $a:expr, b: $b:expr), call a: $($tail:tt)*) => {
        $a(stringify!($($tail)*))
    };

    ((a: $a:expr, b: $b:expr), call b: $($tail:tt)*) => {
        $b(stringify!($($tail)*))
    };

    ($ab:tt, $_skip:tt $($tail:tt)*) => {
        call_a_or_b_on_tail!($ab, $($tail)*)
    };
}

fn compute_len(s: &str) -> usize {
    s.len()
}

fn zero(_s: &str) -> usize {
    0
}

#[derive(Template)]
#[template(path = "rust-macro-args.html")]
struct RustMacrosArgTemplate {}

#[test]
fn args() {
    let template = RustMacrosArgTemplate {};
    assert_eq!("0\n91", template.render().unwrap());
}
