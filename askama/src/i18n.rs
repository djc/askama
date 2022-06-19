use std::collections::HashMap;
use std::iter::FromIterator;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, StaticLoader};
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
    ) -> String {
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
/// The struct is only meant to be used by the [`localization!()`] macro.
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
