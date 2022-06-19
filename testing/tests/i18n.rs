#![cfg(feature = "localization")]

use askama::{langid, Locale, Template};

askama::localization!(LOCALES);

/*
#[derive(Template)]
#[template(path = "i18n_invalid.html")]
struct UsesI18nInvalid<'a> {
    #[locale]
    loc: Locale<'a>,
    name: &'a str,
}
*/

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

/*
#[derive(Template)]
#[template(path = "i18n_broken.html")]
struct InvalidI18n<'a> {
    #[locale]
    loc: Locale<'a>,
    car_color: &'a str,
}
*/

#[test]
fn existing_language() {
    let template = UsesI18n {
        loc: Locale::new(langid!("es-MX"), &LOCALES),
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
fn fallback_language() {
    let template = UsesI18n {
        loc: Locale::new(langid!("nl-BE"), &LOCALES),
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
        loc: Locale::new(langid!("en-US"), &LOCALES),
    };
    assert_eq!(template.render().unwrap(), r#"<h3>This is a test</h3>"#)
}

/*
#[test]
fn invalid_tags_language() {
    let template = InvalidI18n {
        loc: Locale::new(langid!("nl-BE"), &LOCALES),
        car_color: "Red",
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>Unknown localization car</h1>"#
    );
}
*/
