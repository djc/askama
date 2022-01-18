use askama::Template;

const FALSE: &'static bool = &false;

#[derive(Debug, Clone)]
struct User {
    name: String,
    flag: bool,
}

impl User {
    fn ferris() -> Self {
        Self {
            name: "Ferris".to_string(),
            flag: true,
        }
    }
}

#[derive(Template)]
#[template(
    source = r#"Hello {{ user_opt.map(|user| user.name.as_str()).unwrap_or("World") }}"#,
    ext = "txt"
)]
struct ClosureTemplate<'a> {
    user_opt: Option<&'a User>,
}

#[test]
fn test_closure() {
    let user = User::ferris();
    let t = ClosureTemplate {
        user_opt: Some(&user),
    };
    assert_eq!(t.render().unwrap(), "Hello Ferris");

    let t = ClosureTemplate { user_opt: None };
    assert_eq!(t.render().unwrap(), "Hello World");
}

#[derive(Template)]
#[template(
    source = r#"Hello {{ user.map(|user| user.name.as_str()).unwrap_or("World") }}"#,
    ext = "txt"
)]
struct ClosureShadowTemplate<'a> {
    user: Option<&'a User>,
}

#[test]
fn test_closure_shadow() {
    let user = User::ferris();
    let t = ClosureShadowTemplate { user: Some(&user) };
    assert_eq!(t.render().unwrap(), "Hello Ferris");

    let t = ClosureShadowTemplate { user: None };
    assert_eq!(t.render().unwrap(), "Hello World");
}

#[derive(Template)]
#[template(
    source = r#"{{ user_opt.map(|user| user.flag).unwrap_or(FALSE) }}"#,
    ext = "txt"
)]
struct ClosureBorrowTemplate<'a> {
    user_opt: Option<&'a User>,
}

#[test]
fn test_closure_borrow() {
    let user = User::ferris();
    let t = ClosureBorrowTemplate {
        user_opt: Some(&user),
    };
    assert_eq!(t.render().unwrap(), "true");

    let t = ClosureBorrowTemplate { user_opt: None };
    assert_eq!(t.render().unwrap(), "false");
}
