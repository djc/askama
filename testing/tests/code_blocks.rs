use askama::Template;

#[test]
fn test_code1() {
    #[derive(Template)]
    #[template(source = "<\n{%% writer.write_str(self.0)?; %%}\n>", ext = "txt")]
    struct Code<'a>(&'a str);

    assert_eq!(Code("Hello").render().unwrap(), "<\nHello\n>");
}

#[test]
fn test_code1_trim_start() {
    #[derive(Template)]
    #[template(source = "<\n{%%- writer.write_str(self.0)?; %%}\n>", ext = "txt")]
    struct Code<'a>(&'a str);

    assert_eq!(Code("Hello").render().unwrap(), "<Hello\n>");
}

#[test]
fn test_code1_trim_end() {
    #[derive(Template)]
    #[template(source = "<\n{%% writer.write_str(self.0)?; -%%}\n>", ext = "txt")]
    struct Code<'a>(&'a str);

    assert_eq!(Code("Hello").render().unwrap(), "<\nHello>");
}

#[test]
fn test_code1_trim() {
    #[derive(Template)]
    #[template(source = "<\n{%%- writer.write_str(self.0)?; -%%}\n>", ext = "txt")]
    struct Code<'a>(&'a str);

    assert_eq!(Code("Hello").render().unwrap(), "<Hello>");
}

#[test]
fn test_code4() {
    #[derive(Template)]
    #[template(source = "<\n{%%%%% writer.write_str(self.0)?; %%%%%}\n>", ext = "txt")]
    struct Code<'a>(&'a str);

    assert_eq!(Code("Hello").render().unwrap(), "<\nHello\n>");
}

#[test]
fn test_code4_trim() {
    #[derive(Template)]
    #[template(
        source = "<\n{%%%%%- writer.write_str(self.0)?; -%%%%%}\n>",
        ext = "txt"
    )]
    struct Code<'a>(&'a str);

    assert_eq!(Code("Hello").render().unwrap(), "<Hello>");
}

#[test]
fn test_inser_percents() {
    #[derive(Template)]
    #[template(source = r#"< {%%%%% writer.write_str("%%%")?; %%%%%} >"#, ext = "txt")]
    struct Code;

    assert_eq!(Code.render().unwrap(), "< %%% >");
}
