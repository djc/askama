#![cfg(feature = "with-i18n")]
#![allow(unused)]

use askama::{impl_localize, Template};

impl_localize! {
    #[localize(path = "i18n-basic", default_locale = "en-US")]
    struct BasicLocalizer(_);
}

//
#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    name: &'a str,
    hours: f32,
}
