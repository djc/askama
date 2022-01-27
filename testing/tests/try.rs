use askama::Template;

#[derive(Template)]
#[template(source = "{% let v = self.parse()? %}{{s}}={{v}}", ext = "txt")]
struct IntParserTemplate<'a> {
    s: &'a str,
}

impl IntParserTemplate<'_> {
    fn parse(&self) -> Result<i32, std::num::ParseIntError> {
        self.s.parse()
    }
}

#[test]
fn test_int_parser() {
    let template = IntParserTemplate { s: "💯" };
    assert!(matches!(template.render(), Err(askama::Error::Custom(_))));

    let template = IntParserTemplate { s: "100" };
    assert_eq!(template.render().unwrap(), "100=100");
}

#[derive(Template)]
#[template(source = "{{ value()? }}", ext = "txt")]
struct FailFmt {
    value: fn() -> Result<&'static str, std::fmt::Error>,
}

#[test]
fn fail_fmt() {
    let template = FailFmt {
        value: || Err(std::fmt::Error),
    };
    assert!(matches!(template.render(), Err(askama::Error::Fmt(_))));

    let template = FailFmt {
        value: || Ok("hello world"),
    };
    assert_eq!(template.render().unwrap(), "hello world");
}

#[derive(Template)]
#[template(source = "{{ value()? }}", ext = "txt")]
struct FailStr {
    value: fn() -> Result<&'static str, &'static str>,
}

#[test]
fn fail_str() {
    let template = FailStr {
        value: || Err("FAIL"),
    };
    assert!(matches!(template.render(), Err(askama::Error::Custom(_))));
    assert_eq!(format!("{}", &template.render().unwrap_err()), "FAIL");

    let template = FailStr {
        value: || Ok("hello world"),
    };
    assert_eq!(template.render().unwrap(), "hello world");
}
