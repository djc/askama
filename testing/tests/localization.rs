use askama::Template;

#[derive(Template)]
#[template(path = "localization/index.html", localizer = "language")]
#[l10n(pattern = r#""de""#, path = "localization/index.de.html")]
#[l10n(pattern = r#""en""#, path = "localization/index.en.html")]
#[l10n(pattern = r#""es""#, path = "localization/index.es.html")]
#[l10n(pattern = r#""fr""#, path = "localization/index.fr.html")]
struct LetDestructoringTuple<'a> {
    language: &'a str,
    user: &'a str,
}

#[test]
fn test_localization() {
    let template = LetDestructoringTuple {
        language: "de",
        user: "you",
    };
    assert_eq!(template.render().unwrap(), "Hallo, you!");

    let template = LetDestructoringTuple {
        language: "en",
        user: "you",
    };
    assert_eq!(template.render().unwrap(), "Hello, you!");

    let template = LetDestructoringTuple {
        language: "es",
        user: "you",
    };
    assert_eq!(template.render().unwrap(), "¡Hola, you!");

    let template = LetDestructoringTuple {
        language: "fr",
        user: "you",
    };
    assert_eq!(
        template.render().unwrap(),
        "Localization test:\nBonjour, you !"
    );

    let template = LetDestructoringTuple {
        language: "xx",
        user: "you",
    };
    assert_eq!(template.render().unwrap(), "Not implemented: xx");
}
