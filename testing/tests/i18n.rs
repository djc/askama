// TODO
/*
#![cfg(feature = "with-i18n")]
#![allow(unused)]
*/
#![cfg(feature = "localization")]
use askama::init_translation;
use askama::Template;

init_translation! {
    pub MyLocalizer {
        static_loader_name: LOCALES,
        locales: "i18n-basic",
        fallback_language: "en-US",
        customise: |bundle| bundle.set_use_isolating(false)
    }
}

#[derive(Template)]
#[template(path = "i18n_invalid.html")]
struct UsesI18nInvalid<'a> {
    #[locale]
    loc: MyLocalizer,
    name: &'a str,
}

#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    #[locale]
    loc: MyLocalizer,
    name: &'a str,
    hours: f64,
}
#[derive(Template)]
#[template(path = "i18n_no_args.html")]
struct UsesNoArgsI18n<'a> {
    #[locale]
    loc: MyLocalizer,
    test: &'a str,
}

#[test]
fn existing_language() {
    let template = UsesI18n {
        loc: MyLocalizer::new(unic_langid::langid!("es-MX")),
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
        loc: MyLocalizer::new(unic_langid::langid!("nl-BE")),
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
        loc: MyLocalizer::new(unic_langid::langid!("es-MX")),
        test: ""
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h3>This is a test</h3>"#
    )
}
