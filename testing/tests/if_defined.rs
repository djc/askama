use askama::Template;

#[derive(Template)]
#[template(path = "if-defined.html")]
struct IfDefined<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "if-defined.html")]
struct IfNotDefined {
    cond: bool,
}

#[derive(Template)]
#[template(path = "if-defined.html")]
struct IfOtherDefined<'a> {
    cond: bool,
    other_name: &'a str,
}

#[test]
fn test_if_defined() {
    let t = IfDefined { name: "Alice" };
    assert_eq!(t.render().unwrap(), "Hello Alice!");
    let t = IfNotDefined { cond: false };
    assert_eq!(t.render().unwrap(), "!cond");
    let t = IfNotDefined { cond: true };
    assert_eq!(t.render().unwrap(), "Both names aren't defined");
    let t = IfOtherDefined {
        cond: true,
        other_name: "Bob",
    };
    assert_eq!(t.render().unwrap(), "Aloha Bob!");
}

#[derive(Template)]
#[template(path = "include-if-defined.html")]
struct IncludeIfDefined<'a> {
    cond: bool,
    my_name: Option<&'a str>,
    my_other_name: Option<&'a str>,
}

#[test]
fn test_include_if_defined() {
    let t = IncludeIfDefined {
        cond: false,
        my_name: Some("Alice"),
        my_other_name: None,
    };
    assert_eq!(t.render().unwrap(), "Hello Alice!");
    let t = IncludeIfDefined {
        cond: true,
        my_name: None,
        my_other_name: Some("Bob"),
    };
    assert_eq!(t.render().unwrap(), "Aloha Bob!");
    let t = IncludeIfDefined {
        cond: true,
        my_name: None,
        my_other_name: None,
    };
    assert_eq!(t.render().unwrap(), "Both names aren't defined");
}
