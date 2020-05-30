use askama::Template;

#[derive(Template)]
#[template(path = "render_in_place.html")]
struct RenderInPlace<'a> {
    s1: SectionOne<'a>,
    s2: SectionTwo<'a>,
    s3: &'a Vec<SectionOne<'a>>,
}

#[derive(Template)]
#[template(source = "A={{ a }}\nB={{ b }}", ext = "html")]
struct SectionOne<'a> {
    a: &'a str,
    b: &'a str,
}

#[derive(Template)]
#[template(source = "C={{ c }}\nD={{ d }}", ext = "html")]
struct SectionTwo<'a> {
    c: &'a str,
    d: &'a str,
}

#[test]
fn test_render_in_place() {
    let t = RenderInPlace {
        s1: SectionOne { a: "A", b: "B" },
        s2: SectionTwo { c: "C", d: "D" },
        s3: &vec![
            SectionOne { a: "1", b: "2" },
            SectionOne { a: "A", b: "B" },
            SectionOne { a: "a", b: "b" },
        ],
    };
    assert_eq!(
        t.render().unwrap(),
       "Section 1: A=A\nB=B\nSection 2: C=C\nD=D\nSection 3 for:\n* A=1\nB=2\n* A=A\nB=B\n* A=a\nB=b\n"
    );
}
