#![cfg(feature = "with-i18n")]
#![allow(unused)]

use askama::{impl_localize, Localize, Template};

impl_localize! {
    #[localize(path = "i18n-basic", default_locale = "en_US")]
    struct BasicLocalizer(_);
}

#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    #[localizer]
    loc: BasicLocalizer,
    name: &'a str,
    hours: f32,
}

#[test]
fn basic() {
    let template = UsesI18n {
        loc: BasicLocalizer::new(Some("es_MX"), None),
        name: "Hilda",
        hours: 300072.3,
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>¡Hola, Hilda!</h1>
<h3>Tienes 300072.3 horas.</h3>"#
    )
}