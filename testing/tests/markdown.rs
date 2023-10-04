#![cfg(feature = "markdown")]

use askama::Template;
use comrak::Options;

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
#[template(
    source = "{{before}}{{content|markdown(options)}}{{after}}",
    ext = "html"
)]
struct MarkdownWithOptionsTemplate<'a> {
    before: &'a str,
    after: &'a str,
    content: &'a str,
    options: &'a Options,
}

#[test]
fn test_markdown_with_options() {
    let mut options = Options::default();
    options.render.unsafe_ = true;

    let s = MarkdownWithOptionsTemplate {
        before: "before",
        after: "after",
        content: "* 1\n* <script>alert('Lol, hacked!')</script>\n* 3",
        options: &options,
    };
    assert_eq!(
        s.render().unwrap(),
        "\
before\
<ul>\n\
<li>1</li>\n\
<li>\n\
<script>alert('Lol, hacked!')</script>\n\
</li>\n\
<li>3</li>\n\
</ul>\n\
after",
    );
}
