use askama::Template;

#[derive(Template)]
#[template(source = "{{ self::simple_test_fn() }}", ext = "txt")]
struct FnTemplate;

fn simple_test_fn() -> &'static str {
    "foo"
}

#[test]
fn test_fn() {
    let t = FnTemplate;
    assert_eq!(t.render().unwrap(), "foo");
}


#[derive(Template)]
#[template(source = "{{ self::arg_test_fn(arg) }}", ext = "txt")]
struct FnArgsTemplate{
    arg: &'static str
}

fn arg_test_fn(s: &'static str) -> String {
    format!("{}!", s)
}

#[test]
fn test_fn_args() {
    let t = FnArgsTemplate { arg: "foo" };
    assert_eq!(t.render().unwrap(), "foo!");
}


#[derive(Template)]
#[template(source = "{% if self::if_test_fn() %}foo{% endif %}", ext = "txt")]
struct FnIfTemplate;

fn if_test_fn() -> bool { true }

#[test]
fn test_fn_if() {
    let t = FnIfTemplate;
    assert_eq!(t.render().unwrap(), "foo");
}

