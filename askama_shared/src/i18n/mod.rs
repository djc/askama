//! Module `i18n` provides tools used by askama's internationalization /
//! localization system, which you can use to translate your templates into other languages.

pub type I18nValue = fluent_bundle::FluentValue;

/// Types and functions used in the implementation of `#[derive(Localizer)]`. You shouldn't ever need
/// to use these types directly.
/// In general, the policy is to move as much code as possible into here; whatever absolutely *must*
/// be included in the generated code is in the quote! block in `askama_derive`.
pub mod macro_impl;
