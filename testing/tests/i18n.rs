#![cfg(feature = "with-i18n")]
#![allow(unused)]

mod i18n {
    askama::init_askama_i18n! {"i18n-basic"}
}

use askama::Template;

#[derive(Template)]
#[template(path = "i18n.html")]
struct UsesI18n<'a> {
    name: &'a str,
    hours: f32,
}
