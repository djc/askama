use askama::Template;

#[derive(Template)]
#[template(path = "foo.html")]
struct PathHtml;

#[test]
fn test_path_ext_html() {
    let t = PathHtml;
    assert_eq!(t.render().unwrap(), "foo.html");
    assert_eq!(t.extension(), Some("html"));
}

#[derive(Template)]
#[template(path = "foo.jinja")]
struct PathJinja;

#[test]
fn test_path_ext_jinja() {
    let t = PathJinja;
    assert_eq!(t.render().unwrap(), "foo.jinja");
    assert_eq!(t.extension(), Some("jinja"));
}

#[derive(Template)]
#[template(path = "foo.html.jinja")]
struct PathHtmlJinja;

#[test]
fn test_path_ext_html_jinja() {
    let t = PathHtmlJinja;
    assert_eq!(t.render().unwrap(), "foo.html.jinja");
    assert_eq!(t.extension(), Some("html"));
}

#[derive(Template)]
#[template(path = "foo.html", ext = "txt")]
struct PathHtmlAndExtTxt;

#[test]
fn test_path_ext_html_and_ext_txt() {
    let t = PathHtmlAndExtTxt;
    assert_eq!(t.render().unwrap(), "foo.html");
    assert_eq!(t.extension(), Some("txt"));
}

#[derive(Template)]
#[template(path = "foo.jinja", ext = "txt")]
struct PathJinjaAndExtTxt;

#[test]
fn test_path_ext_jinja_and_ext_txt() {
    let t = PathJinjaAndExtTxt;
    assert_eq!(t.render().unwrap(), "foo.jinja");
    assert_eq!(t.extension(), Some("txt"));
}

#[derive(Template)]
#[template(path = "foo.html.jinja", ext = "txt")]
struct PathHtmlJinjaAndExtTxt;

#[test]
fn test_path_ext_html_jinja_and_ext_txt() {
    let t = PathHtmlJinjaAndExtTxt;
    assert_eq!(t.render().unwrap(), "foo.html.jinja");
    assert_eq!(t.extension(), Some("txt"));
}
