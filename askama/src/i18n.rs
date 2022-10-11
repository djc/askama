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
//! struct UsesI18n<'a> {
//!     #[locale]
//!     loc: Locale<'a>,
//!     name: &'a str,
//! }
//!
//! let template = UsesI18n {
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
use parking_lot::const_mutex;

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
        text_id: &str,
        args: impl IntoIterator<Item = (&'a str, FluentValue<'a>)>,
    ) -> Option<String> {
        let args = HashMap::<&str, FluentValue<'_>>::from_iter(args);
        let args = match args.is_empty() {
            true => None,
            false => Some(&args),
        };
        self.loader.lookup_complete(&self.language, text_id, args)
    }
}

/// Similar to OnceCell, but it has an additional take() function, which can only be used once,
/// and only if the instance was never dereferenced.
///
/// The struct is only meant to be used by the [`i18n_load!()`] macro.
/// Concurrent access will deliberately panic.
///
/// Rationale: StaticLoader cannot be cloned.
#[doc(hidden)]
pub struct Unlazy<T>(parking_lot::Mutex<UnlazyEnum<T>>);

enum UnlazyEnum<T> {
    Generator(Option<fn() -> T>),
    Value(Box<T>),
}

impl<T> Unlazy<T> {
    pub const fn new(f: fn() -> T) -> Self {
        Self(const_mutex(UnlazyEnum::Generator(Some(f))))
    }

    pub fn take(&self) -> T {
        let f = match &mut *self.0.try_lock().unwrap() {
            UnlazyEnum::Generator(f) => f.take(),
            _ => None,
        };
        f.unwrap()()
    }
}

impl<T> std::ops::Deref for Unlazy<T>
where
    Self: 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let data = &mut *self.0.try_lock().unwrap();
        let value: &T = match data {
            UnlazyEnum::Generator(f) => {
                *data = UnlazyEnum::Value(Box::new(f.take().unwrap()()));
                match data {
                    UnlazyEnum::Value(value) => value,
                    _ => unreachable!(),
                }
            }
            UnlazyEnum::Value(value) => value,
        };

        // SAFETY: This transmutation is safe because once a value is assigned,
        //         it won't be unassigned again, and Self has static lifetime.
        unsafe { std::mem::transmute(value) }
    }
}
