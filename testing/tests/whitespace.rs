use askama::Template;

#[derive(askama::Template, Default)]
#[template(path = "allow-whitespaces.html")]
struct AllowWhitespaces {
    tuple: (u64, u64, u64, u64),
    string: &'static str,
    option: Option<bool>,
    nested_1: AllowWhitespacesNested1,
}

#[derive(Default)]
struct AllowWhitespacesNested1 {
    nested_2: AllowWhitespacesNested2,
}

#[derive(Default)]
struct AllowWhitespacesNested2 {
    array: &'static [&'static str],
    hash: std::collections::HashMap<&'static str, &'static str>,
}

impl AllowWhitespaces {
    fn f0(&self) -> &str {
        ""
    }
    fn f1(&self, _a: &str) -> &str {
        ""
    }
    fn f2(&self, _a: &str, _b: &str) -> &str {
        ""
    }
}

#[test]
fn test_extra_whitespace() {
    let mut template = AllowWhitespaces::default();
    template.nested_1.nested_2.array = &["a0", "a1", "a2", "a3"];
    template.nested_1.nested_2.hash.insert("key", "value");
    assert_eq!(template.render().unwrap(), "\n0\n0\n0\n0\n\n\n\n0\n0\n0\n0\n0\n\na0\na1\nvalue\n\n\n\n\n\n[\n  \"a0\",\n  \"a1\",\n  \"a2\",\n  \"a3\"\n]\n[\n  \"a0\",\n  \"a1\",\n  \"a2\",\n  \"a3\"\n][\n  \"a0\",\n  \"a1\",\n  \"a2\",\n  \"a3\"\n]\n[\n  \"a1\"\n][\n  \"a1\"\n]\n[\n  \"a1\",\n  \"a2\"\n][\n  \"a1\",\n  \"a2\"\n]\n[\n  \"a1\"\n][\n  \"a1\"\n]1-1-1\n3333 3\n2222 2\n0000 0\n3333 3\n\ntruefalse\nfalsefalsefalse\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
}

#[derive(askama::Template)]
#[template(source = " a \t b   c\n\nd e\n  f\n\n\n", ext = "txt", strip = "none")]
struct StripNone;

#[test]
fn test_strip_none() {
    assert_eq!(StripNone.render().unwrap(), " a \t b   c\n\nd e\n  f");
}

#[derive(askama::Template)]
#[template(source = " a \t b   c\n\nd e\n  f\n\n\n", ext = "txt", strip = "tail")]
struct StripTail;

#[test]
fn test_strip_tail() {
    assert_eq!(StripTail.render().unwrap(), " a \t b   c\n\nd e\n  f");
}

#[derive(askama::Template)]
#[template(
    source = " a \t b   c\n\nd e\n  f\n\n\n",
    ext = "txt",
    strip = "trim-lines"
)]
struct StripTrimLines;

#[test]
fn test_strip_trim_lines() {
    assert_eq!(StripTrimLines.render().unwrap(), "a \t b   c\nd e\nf");
}

#[derive(askama::Template)]
#[template(source = " a \t b   c\n\nd e\n  f\n\n\n", ext = "txt", strip = "eager")]
struct StripEager;

#[test]
fn test_strip_eager() {
    assert_eq!(StripEager.render().unwrap(), "a b c\nd e\nf");
}

#[derive(askama::Template)]
#[template(path = "whitespace_trimming.html", strip = "none")]
struct StripNone2;

#[test]
fn test_strip_none2() {
    assert_eq!(
        StripNone2.render().unwrap(),
        "\n<!DOCTYPE html>\n\n<html>\n <body>\n  <p>\n   .  .   .\n  </p>\n </body>\n</html>"
    );
}

#[derive(askama::Template)]
#[template(path = "whitespace_trimming.html", strip = "tail")]
struct StripTail2;

#[test]
fn test_strip_tail2() {
    assert_eq!(
        StripTail2.render().unwrap(),
        "\n<!DOCTYPE html>\n\n<html>\n <body>\n  <p>\n   .  .   .\n  </p>\n </body>\n</html>"
    );
}

#[derive(askama::Template)]
#[template(path = "whitespace_trimming.html", strip = "trim-lines")]
struct StripTrimLines2;

#[test]
fn test_strip_trim_lines2() {
    assert_eq!(
        StripTrimLines2.render().unwrap(),
        "<!DOCTYPE html>\n<html>\n<body>\n<p>\n.  .   .\n</p>\n</body>\n</html>"
    );
}

#[derive(askama::Template)]
#[template(path = "whitespace_trimming.html", strip = "eager")]
struct StripEager2;

#[test]
fn test_strip_eager2() {
    assert_eq!(
        StripEager2.render().unwrap(),
        "<!DOCTYPE html>\n<html>\n<body>\n<p>\n. . .\n</p>\n</body>\n</html>"
    );
}

#[test]
fn test_strip_class() {
    #[derive(askama::Template)]
    #[template(
        source = r#"<li{% if self.0 %}class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass00(bool);
    assert_eq!(StripClass00(false).render().unwrap(), "<li>");
    assert_eq!(
        StripClass00(true).render().unwrap(),
        r#"<liclass="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li{% if self.0 %} class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass01(bool);
    assert_eq!(StripClass01(false).render().unwrap(), "<li>");
    assert_eq!(
        StripClass01(true).render().unwrap(),
        r#"<li class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li{% if self.0 %}  class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass02(bool);
    assert_eq!(StripClass02(false).render().unwrap(), "<li>");
    assert_eq!(
        StripClass02(true).render().unwrap(),
        r#"<li class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li {% if self.0 %}class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass10(bool);
    assert_eq!(StripClass10(false).render().unwrap(), "<li >");
    assert_eq!(
        StripClass10(true).render().unwrap(),
        r#"<li class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li {% if self.0 %} class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass11(bool);
    assert_eq!(StripClass11(false).render().unwrap(), "<li >");
    assert_eq!(
        StripClass11(true).render().unwrap(),
        r#"<li  class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li {% if self.0 %}  class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass12(bool);
    assert_eq!(StripClass12(false).render().unwrap(), "<li >");
    assert_eq!(
        StripClass12(true).render().unwrap(),
        r#"<li  class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li  {% if self.0 %}class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass20(bool);
    assert_eq!(StripClass20(false).render().unwrap(), "<li >");
    assert_eq!(
        StripClass20(true).render().unwrap(),
        r#"<li class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li  {% if self.0 %} class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass21(bool);
    assert_eq!(StripClass21(false).render().unwrap(), "<li >");
    assert_eq!(
        StripClass21(true).render().unwrap(),
        r#"<li  class="active">"#
    );

    #[derive(askama::Template)]
    #[template(
        source = r#"<li  {% if self.0 %}  class="active"{% endif %}>"#,
        ext = "txt",
        strip = "eager"
    )]
    struct StripClass22(bool);
    assert_eq!(StripClass22(false).render().unwrap(), "<li >");
    assert_eq!(
        StripClass22(true).render().unwrap(),
        r#"<li  class="active">"#
    );
}
