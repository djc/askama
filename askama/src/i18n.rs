//! Module for compile time checked localization
//!
//! # Example:
//!
//! [Fluent Translation List](https://projectfluent.org/) resource file `i18n/es-MX/basic.ftl`:
//!
//! ```ftl
//! greeting = ¡Hola, { $name }!
//! ```
//!
//! Askama HTML template `templates/example.html`:
//!
//! ```html
//! <h1>{{ localize("greeting", name: name) }}</h1>
//! ```
//!
//! Rust usage:
//! ```ignore
//! use askama::i18n::{langid, Locale};
//! use askama::Template;
//!
//! askama::i18n::load!(LOCALES);
//!
//! #[derive(Template)]
//! #[template(path = "example.html")]
//! struct ExampleTemplate<'a> {
//!     #[locale]
//!     loc: Locale<'a>,
//!     name: &'a str,
//! }
//!
//! let template = ExampleTemplate {
//!     loc: Locale::new(langid!("es-MX"), &LOCALES),
//!     name: "Hilda",
//! };
//!
//! // "<h1>¡Hola, Hilda!</h1>"
//! template.render().unwrap();
//! ```

use std::collections::HashMap;
use std::iter::FromIterator;

// Re-export conventiently as `askama::i18n::load!()`.
// Proc-macro crates can only export macros from their root namespace.
/// Load locales at compile time. See example above for usage.
pub use askama_derive::i18n_load as load;

pub use fluent_templates::{self, fluent_bundle::FluentValue, fs::langid, LanguageIdentifier};
use fluent_templates::{Loader, StaticLoader};

pub struct Locale<'a> {
    loader: &'a StaticLoader,
    language: LanguageIdentifier,
}

impl Locale<'_> {
    pub fn new(language: LanguageIdentifier, loader: &'static StaticLoader) -> Self {
        Self { loader, language }
    }

    pub fn translate<'a>(
        &self,
        msg_id: &str,
        args: impl IntoIterator<Item = (&'a str, FluentValue<'a>)>,
    ) -> Option<String> {
        let args = HashMap::<&str, FluentValue<'_>>::from_iter(args);
        let args = match args.is_empty() {
            true => None,
            false => Some(&args),
        };
        self.loader.lookup_complete(&self.language, msg_id, args)
    }
}
