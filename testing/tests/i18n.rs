#![cfg(feature = "i18n")]

use askama::i18n::{langid, Locale};
use askama::Template;

askama::i18n::load!(LOCALES);

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
fn test_existing_language() {
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
fn test_fallback_language() {
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
fn test_no_args() {
    let template = UsesNoArgsI18n {
        loc: Locale::new(langid!("en-US"), &LOCALES),
    };
    assert_eq!(template.render().unwrap(), r#"<h3>This is a test</h3>"#)
}
