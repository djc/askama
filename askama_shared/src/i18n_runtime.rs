//! Code used in the implementation of `impl_localize!` and the `localize()` filter.
//!
//! Everything in this module should be considered an internal implementation detail; it is only public
//! for use by the macro.
//!
//! Maintenance note: in general, the policy is to move as much i18n code as possible into here;
//! whatever absolutely *must* be included in the generated code is done in askama_derive.

use accept_language::parse as accept_language_parse;
use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;

use super::{Error, Result};

pub use lazy_static::lazy_static;

/// An I18n argument value. Instantiated only by the `{ localize() }` filter.
pub type I18nValue = fluent_bundle::FluentValue;

/// A known locale. Instantiated only by the `impl_localize!` macro.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Locale(pub &'static str);

/// Sources; an array mapping Locales to fluent source strings. Instantiated only by the `impl_localize!` macro.
pub type Sources = &'static [(Locale, &'static str)];

/// Sources that have been parsed. Instantiated only by the `impl_localize!` macro.
///
/// This type is initialized in a lazy_static! in impl_localize!,
/// because FluentBundle can only take FluentResources by reference;
/// we have to store them somewhere to reference them.
/// This can go away once https://github.com/projectfluent/fluent-rs/issues/103 lands.
pub struct Resources(Vec<(Locale, FluentResource)>);

impl Resources {
    /// Parse a list of sources into a list of resources.
    pub fn new(sources: Sources) -> Resources {
        Resources(
            sources
                .iter()
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
/// the output executable / library easy. Instantiated only by the `impl_localize!` macro.
pub struct StaticParser<'a> {
    /// Bundles used for localization.
    /// Maps long-form locales (e.g. "en_US", not just "en") to their respective bundles.
    bundles: HashMap<Locale, FluentBundle<'a>>,

    /// A listing of available locales.
    /// Long-form locales map to themselves ("en_US" => [Locale("en_US")]);
    /// Short-form locales map to all available long-form locales, in alphabetical order:
    /// ("en" => [Locale("en_UK"), Locale("en_US")]).
    locales: HashMap<&'static str, Vec<Locale>>,

    /// The default locale chosen if no others can be determined.
    default_locale: Locale,
}

impl<'a> StaticParser<'a> {
    /// Create a StaticParser.
    pub fn new(resources: &'a Resources, default_locale: Locale) -> StaticParser<'a> {
        assert!(
            resources
                .0
                .iter()
                .find(|(locale, _)| *locale == default_locale)
                .is_some(),
            "default locale not available!"
        );

        let mut bundles = HashMap::new();
        let mut locales = HashMap::new();
        for (locale, resource) in resources.0.iter() {
            // confusingly, this value is used by fluent for number and date formatting only.
            // we have to implement looking up missing messages in other bundles ourselves.
            let fallback_chain = &[locale.0];

            let mut bundle = FluentBundle::new(fallback_chain);

            bundle
                .add_resource(resource)
                .expect("failed to add resource");
            bundles.insert(*locale, bundle);
            locales.insert(locale.0, vec![*locale]);

            let short = &locale.0[..2];
            let shorts = locales.entry(short).or_insert_with(|| vec![]);
            shorts.push(*locale);
            // ensure determinism in fallback order
            shorts.sort();
        }

        StaticParser {
            bundles,
            locales,
            default_locale,
        }
    }

    /// Creates a chain of locales to use for message lookups.
    /// * `user_locales`: a list of locales allowed by the user,
    ///   in descending order of preference.
    ///    - May be empty.
    ///    - May be short-form locales (e.g. "en")
    /// * `accept_language`: an `Accept-Language` header, if present.
    pub fn create_locale_chain(
        &self,
        user_locales: &[&str],
        accept_language: Option<&str>,
    ) -> Vec<Locale> {
        let mut chain = vec![];

        // when adding a locale "en_AU", also check its short form "en",
        // and also add all locales that that short form maps to.
        // this ensures that a locale chain like "es-AR", "en-US" will
        // pull messages from "es-MX" before going to english.
        //
        // note: this has the side effect of discarding some ordering information.
        //
        // TODO: discuss whether this is a reasonable approach.
        let mut add = |locale_code: &str| {
            let mut codes = &[locale_code, &locale_code[..2]][..];
            if locale_code.len() == 2 {
                codes = &codes[..1]
            }

            for code in codes {
                if let Some(locales) = self.locales.get(code) {
                    for locale in locales {
                        if !chain.contains(locale) {
                            chain.push(*locale);
                        }
                    }
                }
            }
        };
        for locale_code in user_locales {
            add(locale_code);
        }
        if let Some(accept_language) = accept_language {
            for locale_code in &accept_language_parse(accept_language) {
                add(locale_code);
            }
        }

        if !chain.contains(&self.default_locale) {
            chain.push(self.default_locale);
        }

        chain
    }

    /// Localize a message.
    /// * `locale_chain`: a list of locales, in descending order of preference
    /// * `message`: a message ID
    /// * `args`: a slice of arguments to pass to Fluent.
    pub fn localize(
        &self,
        locale_chain: &[Locale],
        message: &str,
        args: &[(&str, &FluentValue)],
    ) -> Result<String> {
        let args = if args.len() == 0 {
            None
        } else {
            Some(args.into_iter().map(|(k, v)| (*k, (*v).clone())).collect())
        };
        let args = args.as_ref();

        for locale in locale_chain {
            let bundle = self.bundles.get(locale);
            let bundle = if let Some(bundle) = bundle {
                bundle
            } else {
                // TODO warn?
                continue;
            };
            // this API is weirdly awful;
            // format returns Option<(String, Vec<FluentError>)>
            // which we have to cope with
            let result = bundle.format(message, args);

            if let Some((result, errs)) = result {
                if errs.len() == 0 {
                    return Ok(result);
                } else {
                    continue;

                    // TODO: fluent degrades gracefully; maybe just warn here?
                    // Err(Error::I18n(errs.pop().unwrap()))
                }
            }
        }
        // nowhere to fall back to
        Err(Error::NoTranslationsForMessage(format!(
            "no translations for message {} in locale chain {:?}",
            message, locale_chain
        )))
    }

    pub fn has_message(&self, locale_chain: &[Locale], message: &str) -> bool {
        locale_chain
            .iter()
            .flat_map(|locale| self.bundles.get(locale))
            .any(|bundle| bundle.has_message(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCES: Sources = &[
        (
            Locale("en_US"),
            r#"
greeting = Hello, { $name }! You are { $hours } hours old.
goodbye = Goodbye.
"#,
        ),
        (
            Locale("en_AU"),
            r#"
greeting = G'day, { $name }! You are { $hours } hours old.
goodbye = Hooroo.
"#,
        ),
        (
            Locale("es_MX"),
            r#"
greeting = ¡Hola, { $name }! Tienes { $hours } horas.
goodbye = Adiós.
"#,
        ),
        (
            Locale("de_DE"),
            "greeting = Hallo { $name }! Du bist { $hours } Stunden alt.",
        ),
    ];

    #[test]
    fn basic() -> Result<()> {
        let resources = Resources::new(SOURCES);
        let bundles = StaticParser::new(&resources, Locale("en_US"));
        let name = FluentValue::from("Jamie");
        let hours = FluentValue::from(190321.31);
        let args = &[("name", &name), ("hours", &hours)][..];

        assert_eq!(
            bundles.localize(&[Locale("en_US")], "greeting", args)?,
            "Hello, Jamie! You are 190321.31 hours old."
        );
        assert_eq!(
            bundles.localize(&[Locale("es_MX")], "greeting", args)?,
            "¡Hola, Jamie! Tienes 190321.31 horas."
        );
        assert_eq!(
            bundles.localize(&[Locale("de_DE")], "greeting", args)?,
            "Hallo Jamie! Du bist 190321.31 Stunden alt."
        );

        // missing messages should fall back to first available
        assert_eq!(
            bundles.localize(
                &[Locale("de_DE"), Locale("es_MX"), Locale("en_US")],
                "goodbye",
                &[]
            )?,
            "Adiós."
        );

        if let Ok(_) = bundles.localize(&[Locale("en_US")], "bananas", &[]) {
            panic!("Should return Err on missing message");
        }

        Ok(())
    }

    #[test]
    fn create_locale_chain() {
        let resources = Resources::new(SOURCES);
        let bundles = StaticParser::new(&resources, Locale("en_US"));

        // accept-language parser works + short-code lookup works
        assert_eq!(
            bundles.create_locale_chain(&[], Some("en_US, es_MX; q=0.5")),
            &[Locale("en_US"), Locale("en_AU"), Locale("es_MX")]
        );

        // first choice has precedence
        assert_eq!(
            bundles.create_locale_chain(&["es_MX"], Some("en_US; q=0.5")),
            &[Locale("es_MX"), Locale("en_US"), Locale("en_AU")]
        );

        // short codes work
        assert_eq!(
            bundles.create_locale_chain(&[], Some("en")),
            &[Locale("en_AU"), Locale("en_US")]
        );

        // default works
        assert_eq!(bundles.create_locale_chain(&[], None), &[Locale("en_US")]);

        // missing languages fall through to default
        assert_eq!(
            bundles.create_locale_chain(&["zh_HK"], Some("xy_ZW")),
            &[Locale("en_US")]
        );
    }
}
