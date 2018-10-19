#[macro_use]
extern crate askama;

use askama::Template;

#[derive(Template)]
#[template(path = "escape-none.html", escape = "none")]
struct EscapeTemplate {
    pub title: String,
}

#[test]
fn test_escape() {
    let t = EscapeTemplate {
        title: "foo".to_string(),
    };
    assert_eq!(
        t.render().unwrap(),
        "<html>
    <head>
        <title>foo</title>
    </head>
    <body>
    </body>
</html>"
    );
}
