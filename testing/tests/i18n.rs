#![cfg(feature = "with-i18n")]
#![allow(unused)]

use askama::{impl_localize, Localize, Template};

impl_localize! {
    #[localize(path = "i18n-basic", default_locale = "en-US")]
    struct BasicLocalizer(_);
}

#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    #[localizer]
    localizer: BasicLocalizer,
    name: &'a str,
    hours: f32,
}

#[test]
fn basic() {
    let template = UsesI18n {
        localizer: BasicLocalizer::new(Some("es-MX"), None),
        name: "Hilda",
        hours: 300072.3,
    };
    assert_eq!(
        template.render().unwrap(),
        r#"<h1>¡Hola, Hilda!</h1>
<h3>Tienes 300072.3 horas.</h3>"#
    )
}
