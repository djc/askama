// TODO
/*
#![cfg(feature = "with-i18n")]
#![allow(unused)]
*/

use askama::Template;

fluent_templates::static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "i18n-basic",
        // The language to falback on if something is not present.
        fallback_language: "en-US",
        // Optional: A fluent resource that is shared with every locale.
        //core_locales: "/core.ftl",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    #[localizer]
    loc: (
        &'a fluent_templates::StaticLoader,
        &'a unic_langid::LanguageIdentifier,
    ),
    name: &'a str,
    hours: f64,
}

#[test]
fn existing_language() {
    let template = UsesI18n {
        loc: (&LOCALES, &unic_langid::langid!("es-MX")),
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
        loc: (&LOCALES, &unic_langid::langid!("nl-BE")),
        name: "Hilda",
        hours: 300072.3,
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>Hello, Hilda!</h1>
<h3>You are 300072.3 hours old.</h3>"#
    )
}
