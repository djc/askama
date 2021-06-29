use askama::Template;

#[derive(Template)]
#[template(path = "if-let.html")]
struct IfLetTemplate {
    text: Option<&'static str>,
}

#[test]
fn test_if_let() {
    let s = IfLetTemplate {
        text: Some("hello"),
    };
    assert_eq!(s.render().unwrap(), "hello");

    let t = IfLetTemplate { text: None };
    assert_eq!(t.render().unwrap(), "");
}

#[derive(Template)]
#[template(path = "if-let-shadowing.html")]
struct IfLetShadowingTemplate {
    text: Option<&'static str>,
}

#[test]
fn test_if_let_shadowing() {
    let s = IfLetShadowingTemplate {
        text: Some("hello"),
    };
    assert_eq!(s.render().unwrap(), "hello");

    let t = IfLetShadowingTemplate { text: None };
    assert_eq!(t.render().unwrap(), "");
}

struct Digits {
    one: i32,
    two: i32,
    three: i32,
}

#[derive(Template)]
#[template(path = "if-let-struct.html")]
struct IfLetStruct {
    digits: Digits,
}

#[test]
fn test_if_let_struct() {
    let digits = Digits {
        one: 1,
        two: 2,
        three: 3,
    };
    let s = IfLetStruct { digits };
    assert_eq!(s.render().unwrap(), "1 2 3");
}

#[derive(Template)]
#[template(path = "if-let-struct.html")]
struct IfLetStructRef<'a> {
    digits: &'a Digits,
}

#[test]
fn test_if_let_struct_ref() {
    let digits = Digits {
        one: 1,
        two: 2,
        three: 3,
    };
    let s = IfLetStructRef { digits: &digits };
    assert_eq!(s.render().unwrap(), "1 2 3");
}

#[derive(Template)]
#[template(path = "if-let-else.html")]
struct IfLetElse {
    cond: bool,
    value: Result<i32, &'static str>,
}

#[test]
fn test_if_let_else() {
    let s = IfLetElse {
        cond: false,
        value: Ok(4711),
    };
    assert_eq!(s.render().unwrap(), "!cond");

    let s = IfLetElse {
        cond: true,
        value: Ok(4711),
    };
    assert_eq!(s.render().unwrap(), "4711");

    let s = IfLetElse {
        cond: false,
        value: Err("fail"),
    };
    assert_eq!(s.render().unwrap(), "!cond");

    let s = IfLetElse {
        cond: true,
        value: Err("fail"),
    };
    assert_eq!(s.render().unwrap(), "fail");
}
