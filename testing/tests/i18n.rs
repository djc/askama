// TODO
/*
#![cfg(feature = "with-i18n")]
#![allow(unused)]
*/

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
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    #[localizer]
    loc: MyLocalizer,
    name: &'a str,
    hours: f64,
}

#[test]
fn existing_language() {
    let template = UsesI18n {
        loc: MyLocalizer::new(unic_langid::langid!("es-MX"), &LOCALES),
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
        loc: MyLocalizer::new(unic_langid::langid!("nl-BE"), &LOCALES),
        name: "Hilda",
        hours: 300072.3,
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>Hello, Hilda!</h1>
<h3>You are 300072.3 hours old.</h3>"#
    )
}
