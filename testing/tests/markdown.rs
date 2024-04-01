#![cfg(feature = "markdown")]

use askama::Template;

#[derive(Template)]
#[template(source = "{{before}}{{content|markdown}}{{after}}", ext = "html")]
struct MarkdownTemplate<'a> {
    before: &'a str,
    after: &'a str,
    content: &'a str,
}

#[test]
fn test_markdown() {
    let s = MarkdownTemplate {
        before: "before",
        after: "after",
        content: "* 1\n* <script>alert('Lol, hacked!')</script>\n* 3",
    };
    assert_eq!(
        s.render().unwrap(),
        "\
before\
<ul>\n\
<li>1</li>\n\
<li>\n\
&lt;script&gt;alert('Lol, hacked!')&lt;/script&gt;\n\
</li>\n\
<li>3</li>\n\
</ul>\n\
after",
    );
}

#[derive(Template)]
#[template(source = "{{content|markdown}}", ext = "html")]
struct MarkdownStringTemplate {
    content: String,
}

// Tests if the markdown filter accepts String
#[test]
fn test_markdown_owned_string() {
    let template = MarkdownStringTemplate {
        content: "The markdown filter _indeed_ works with __String__".into(),
    };
    assert_eq!(
        template.render().unwrap(),
        "<p>The markdown filter <em>indeed</em> works with <strong>String</strong></p>\n"
    )
}
