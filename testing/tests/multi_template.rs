use askama::Template;

#[derive(Template)]
#[template(path = "localization/index.html", multi = "language")]
#[multi_template(
    pattern = r#""de""#,
    path = "localization/index.de.html",
    escaping = "txt"
)]
#[multi_template(pattern = r#""en""#, path = "localization/index.en.html")]
#[multi_template(pattern = r#""es""#, path = "localization/index.es.html")]
#[multi_template(pattern = r#""fr""#, path = "localization/index.fr.html")]
struct MultiTemplate<'a> {
    language: &'a str,
    user: &'a str,
}

#[test]
fn test_localization() {
    let template = MultiTemplate {
        language: "de",
        user: "<you>",
    };
    assert_eq!(template.render().unwrap(), "Hallo, <you>!");

    let template = MultiTemplate {
        language: "en",
        user: "<you>",
    };
    assert_eq!(template.render().unwrap(), "Hello, &lt;you&gt;!");

    let template = MultiTemplate {
        language: "es",
        user: "<you>",
    };
    assert_eq!(template.render().unwrap(), "¡Hola, &lt;you&gt;!");

    let template = MultiTemplate {
        language: "fr",
        user: "<you>",
    };
    assert_eq!(
        template.render().unwrap(),
        "Localization test:\nBonjour, &lt;you&gt; !"
    );

    let template = MultiTemplate {
        language: "xx",
        user: "<you>",
    };
    assert_eq!(template.render().unwrap(), "Not implemented: xx");
}
