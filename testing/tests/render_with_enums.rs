use askama::Template;

enum SectionOneType<'a> {
    Empty(TEmpty),
    Simple(TSecOneSimple<'a>),
    Full(TSecOneFull<'a>),
}
impl<'a> SectionOneType<'a> {
    pub fn render(&self) -> askama::Result<String> {
        match self {
            SectionOneType::Empty(v) => v.render(),
            SectionOneType::Simple(v) => v.render(),
            SectionOneType::Full(v) => v.render(),
        }
    }
}
enum SectionTwoFormat<'a> {
    Html(TSecTwoHtml<'a>),
    Text(TSecTwoText<'a>),
}
impl<'a> SectionTwoFormat<'a> {
    pub fn render(&self) -> askama::Result<String> {
        match self {
            SectionTwoFormat::Html(v) => v.render(),
            SectionTwoFormat::Text(v) => v.render(),
        }
    }
}
#[derive(Template)]
#[template(path = "render_in_place.html")]
struct RenderInPlace<'a> {
    s1: SectionOneType<'a>,
    s2: SectionTwoFormat<'a>,
    s3: &'a Vec<SectionOneType<'a>>,
}

#[derive(Template)]
#[template(source = "", ext = "txt")]
struct TEmpty {}
#[derive(Template)]
#[template(source = "{{ a }}", ext = "txt")]
struct TSecOneSimple<'a> {
    a: &'a str,
}
#[derive(Template)]
#[template(source = "{{ a }}, {{ b }}", ext = "txt")]
struct TSecOneFull<'a> {
    a: &'a str,
    b: &'a str,
}

#[derive(Template)]
#[template(source = "<span>{{ c }}</span><p>{{ d }}</p>", ext = "html")]
struct TSecTwoHtml<'a> {
    c: &'a str,
    d: &'a str,
}
#[derive(Template)]
#[template(source = "{{ c }}, {{ d }}", ext = "txt")]
struct TSecTwoText<'a> {
    c: &'a str,
    d: &'a str,
}

#[test]
fn test_render_with_enums() {
    let t = RenderInPlace {
        s1: SectionOneType::Empty(TEmpty {}),
        s2: SectionTwoFormat::Html(TSecTwoHtml { c: "C", d: "D" }),
        s3: &vec![
            SectionOneType::Empty(TEmpty {}),
            SectionOneType::Simple(TSecOneSimple { a: "A" }),
            SectionOneType::Full(TSecOneFull { a: "A", b: "B" }),
        ],
    };
    assert_eq!(
        t.render().unwrap(),
        "Section 1: \nSection 2: <span>C</span><p>D</p>\nSection 3 for:\n* \n* A\n* A, B\n"
    );
    let t = RenderInPlace {
        s1: SectionOneType::Empty(TEmpty {}),
        s2: SectionTwoFormat::Text(TSecTwoText { c: "C", d: "D" }),
        s3: &vec![
            SectionOneType::Empty(TEmpty {}),
            SectionOneType::Simple(TSecOneSimple { a: "A" }),
            SectionOneType::Full(TSecOneFull { a: "A", b: "B" }),
        ],
    };
    assert_eq!(
        t.render().unwrap(),
        "Section 1: \nSection 2: C, D\nSection 3 for:\n* \n* A\n* A, B\n"
    );
}
