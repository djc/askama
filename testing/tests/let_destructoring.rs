use askama::Template;

#[derive(Template)]
#[template(source = "{% let (a, b, c) = v %}{{a}}{{b}}{{c}}", ext = "txt")]
struct LetDestructoringTuple {
    v: (i32, i32, i32),
}

#[test]
fn test_let_destruct_tuple() {
    let t = LetDestructoringTuple { v: (1, 2, 3) };
    assert_eq!(t.render().unwrap(), "123");
}

struct UnnamedStruct(i32, i32, i32);

#[derive(Template)]
#[template(
    source = "{% let UnnamedStruct(a, b, c) = v %}{{a}}{{b}}{{c}}",
    ext = "txt"
)]
struct LetDestructoringUnnamedStruct {
    v: UnnamedStruct,
}

#[test]
fn test_let_destruct_unnamed_struct() {
    let t = LetDestructoringUnnamedStruct {
        v: UnnamedStruct(1, 2, 3),
    };
    assert_eq!(t.render().unwrap(), "123");
}

#[derive(Template)]
#[template(
    source = "{% let UnnamedStruct(a, b, c) = v %}{{a}}{{b}}{{c}}",
    ext = "txt"
)]
struct LetDestructoringUnnamedStructRef<'a> {
    v: &'a UnnamedStruct,
}

#[test]
fn test_let_destruct_unnamed_struct_ref() {
    let v = UnnamedStruct(1, 2, 3);
    let t = LetDestructoringUnnamedStructRef { v: &v };
    assert_eq!(t.render().unwrap(), "123");
}

struct NamedStruct {
    a: i32,
    b: i32,
    c: i32,
}

#[derive(Template)]
#[template(
    source = "{% let NamedStruct { a, b: d, c } = v %}{{a}}{{d}}{{c}}",
    ext = "txt"
)]
struct LetDestructoringNamedStruct {
    v: NamedStruct,
}

#[test]
fn test_let_destruct_named_struct() {
    let t = LetDestructoringNamedStruct {
        v: NamedStruct { a: 1, b: 2, c: 3 },
    };
    assert_eq!(t.render().unwrap(), "123");
}

#[derive(Template)]
#[template(
    source = "{% let NamedStruct { a, b: d, c } = v %}{{a}}{{d}}{{c}}",
    ext = "txt"
)]
struct LetDestructoringNamedStructRef<'a> {
    v: &'a NamedStruct,
}

#[test]
fn test_let_destruct_named_struct_ref() {
    let v = NamedStruct { a: 1, b: 2, c: 3 };
    let t = LetDestructoringNamedStructRef { v: &v };
    assert_eq!(t.render().unwrap(), "123");
}

mod some {
    pub mod path {
        pub struct Struct<'a>(pub &'a str);
    }
}

#[derive(Template)]
#[template(source = "{% let some::path::Struct(v) = v %}{{v}}", ext = "txt")]
struct LetDestructoringWithPath<'a> {
    v: some::path::Struct<'a>,
}

#[test]
fn test_let_destruct_with_path() {
    let t = LetDestructoringWithPath {
        v: some::path::Struct("hello"),
    };
    assert_eq!(t.render().unwrap(), "hello");
}
