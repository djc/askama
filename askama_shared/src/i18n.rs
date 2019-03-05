//! Module `i18n` provides tools used by askama's internationalization /
//! localization system, which you can use to translate your templates into other languages.

pub type I18nValue = fluent_bundle::FluentValue;

/// Types and functions used in the implementation of `#[derive(Localizer)]`. You shouldn't ever need
/// to use these types directly.
pub mod macro_impl {
    use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
    use std::collections::{HashMap, HashSet};

    use super::super::{Error, Result};

    pub use lazy_static::lazy_static;

    pub type Sources = &'static [(&'static str, &'static str)];

    pub type FallbackChains = &'static [&'static [&'static str]];

    /// Parsed sources.
    pub struct Resources(HashMap<&'static str, FluentResource>);

    impl Resources {
        pub fn new(sources: Sources) -> Resources {
            Resources(
                sources
                    .into_iter()
                    .map(|(locale, source)| {
                        (
                            *locale,
                            FluentResource::try_new(source.to_string())
                                .expect("baked .ftl translation failed to parse"),
                        )
                    })
                    .collect(),
            )
        }
    }

    /// StaticParser is a type that handles accessing the translations baked into
    /// the output executable / library easy.
    /// Users should never need to interact with it; all uses are through the
    /// `init_askama_i18n!()` macro or codegen for the `localize(...)` filter.
    pub struct StaticParser<'a> {
        bundles: HashMap<&'static str, FluentBundle<'a>>,
        locales: HashSet<&'static str>,
        default_locale: &'static str,
    }

    impl<'a> StaticParser<'a> {
        pub fn new(
            resources: &'a Resources,
            fallback_chains: FallbackChains,
            default_locale: &'static str,
        ) -> StaticParser<'a> {
            assert!(
                resources.0.contains_key(default_locale),
                "default locale not in available languages!"
            );

            let mut bundles = HashMap::new();
            let mut locales = HashSet::new();
            for (locale, resource) in resources.0.iter() {
                let default_chain = &[*locale];

                let chain = fallback_chains
                    .iter()
                    .map(|chain| *chain)
                    .find(|chain| chain[0] == *locale)
                    .unwrap_or(default_chain);

                let mut bundle = FluentBundle::new(chain);

                bundle
                    .add_resource(resource)
                    .expect("failed to add resource");
                bundles.insert(*locale, bundle);
                locales.insert(*locale);
            }

            StaticParser {
                bundles,
                locales,
                default_locale,
            }
        }

        /// Chooses a locale; see the documentation of `new` on the `Localize` trait.
        /// Can return a `'static str` because all available locales are baked into the
        /// output binary.
        pub fn choose_locale(
            &self,
            locale: Option<&str>,
            _accepts_language: Option<&str>,
        ) -> &'static str {
            if let Some(locale) = locale {
                if let Some(&static_locale) = self.locales.get(locale) {
                    return static_locale;
                }
            }
            // TODO: parse accepts_language
            self.default_locale
        }

        pub fn localize(
            &self,
            locale: &str,
            message: &str,
            args: &[(&str, &FluentValue)],
        ) -> Result<String> {
            let bundle = self.bundles.get(locale).unwrap_or_else(|| {
                // TODO: use fallback chains here? might be confusing, could just error
                &self.bundles["en-US"]
            });

            let args = if args.len() == 0 {
                None
            } else {
                // TODO this is an extra copy:
                // remove once fluent has been refactored
                Some(args.into_iter().map(|(k, v)| (*k, (*v).clone())).collect())
            };
            let args = args.as_ref();

            // this API is weirdly awful;
            // format returns Option<(String, Vec<FluentError>)>
            // which we have to cope with
            let result = bundle.format(message, args);

            if let Some((result, mut errs)) = result {
                if errs.len() > 0 {
                    // TODO handle more than 1 error
                    Err(Error::I18n(Some(errs.pop().unwrap())))
                } else {
                    Ok(result)
                }
            } else {
                // TODO better error message here, this shows up as Err(I18n(None)) w/ no explanation
                // in panics
                // TODO find error for missing localizations and fall back to default_locale
                Err(Error::I18n(None))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        const SOURCES: Sources = &[
            (
                "en-US",
                "greeting = Hello, { $name }! You are { $hours } hours old.",
            ),
            (
                "es-MX",
                "greeting = ¡Hola, { $name }! Tienes { $hours } horas.",
            ),
        ];
        const FALLBACK_CHAINS: FallbackChains = &[&["en-US", "en-UK"]];

        #[test]
        fn basic() -> Result<()> {
            let resources = Resources::new(SOURCES);
            let bundles = StaticParser::new(&resources, FALLBACK_CHAINS);
            let mut args = HashMap::new();
            args.insert("name", FluentValue::from("Jamie"));
            args.insert("hours", FluentValue::from(190321.31)); // about 21 years
            let args = Some(&args);

            assert_eq!(
                bundles.localize("en-US", "greeting", args)?,
                "Hello, Jamie! You are 190321.31 hours old."
            );
            assert_eq!(
                bundles.localize("es-MX", "greeting", args)?,
                "¡Hola, Jamie! Tienes 190321.31 horas."
            );

            // missing locales should use english (for now)
            assert_eq!(
                bundles.localize("zh-HK", "greeting", args)?,
                "Hello, Jamie! You are 190321.31 hours old."
            );

            if let Ok(_) = bundles.localize("en-US", "bananas", None) {
                panic!("Should return Err on missing message");
            }

            Ok(())
        }
    }
}
