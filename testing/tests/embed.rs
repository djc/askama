use askama::Template;

struct FakeUser {
    name: String,
}

#[derive(Template)]
#[template(path = "embed_parent.html")]
struct EmbedTemplate {
    user: FakeUser,
}

fn strip_whitespaces(string: &str) -> String {
    string
        .split_whitespace()
        .filter(|char| !char.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end()
        .trim_start()
        .to_string()
}

#[test]
fn test_embed() {
    let expected = strip_whitespaces(
        r#"
    <body>
    <div>
        <h1>Hello Yannik</h1>
        <p>Welcome to this example!</p>
    </div>
    </body>"#,
    );
    let template = EmbedTemplate {
        user: FakeUser {
            name: String::from("Yannik"),
        },
    };
    let rendered = strip_whitespaces(&template.render().unwrap());

    assert_eq!(rendered, expected);
}
