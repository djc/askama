#![cfg(feature = "localization")]
use askama::Template;
use askama::Locale;
use fluent_templates::static_loader;
static_loader! {
    static LOCALES = {
        locales: "i18n-basic",
        fallback_language: "en-US",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}
#[derive(Template)]
#[template(path = "i18n_invalid.html")]
struct UsesI18nInvalid<'a> {
    #[locale]
    loc: Locale<'a>,
    name: &'a str,
}

#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    #[locale]
    loc: Locale<'a>,
    name: &'a str,
    hours: f64,
}
#[derive(Template)]
#[template(path = "i18n_no_args.html")]
struct UsesNoArgsI18n<'a> {
    #[locale]
    loc: Locale<'a>,
}

#[test]
fn existing_language() {
    let template = UsesI18n {
        loc: Locale::new(unic_langid::langid!("es-MX"), &LOCALES),
        name: "Hilda",
        hours: 300072.3,
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>¡Hola, Hilda!</h1>
<h3>Tienes 300072.3 horas.</h3>"#
    )
}

#[test]
fn unknown_language() {
    let template = UsesI18n {
        loc: Locale::new(unic_langid::langid!("nl-BE"), &LOCALES),
        name: "Hilda",
        hours: 300072.3,
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>Hello, Hilda!</h1>
<h3>You are 300072.3 hours old.</h3>"#
    )
}

#[test]
fn no_args() {
    let template = UsesNoArgsI18n {
        loc: Locale::new(unic_langid::langid!("en-US"), &LOCALES),
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h3>This is a test</h3>"#
    )
}
