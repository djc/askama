//! Code used in the implementation of `impl_localize!` and the `localize()` filter.
//!
//! Everything in this module should be considered an internal implementation detail; it is only public
//! for use by the macro.
//!
//! Maintenance note: in general, the policy is to move as much i18n code as possible into here;
//! whatever absolutely *must* be included in the generated code is done in askama_derive.

use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use fluent_locale::{negotiate_languages, parse_accepted_languages, NegotiationStrategy};
use std::collections::{HashMap, HashSet};

use super::{Error, Result};

pub use lazy_static::lazy_static;

/// StaticParser is a type that handles accessing the translations baked into
/// the output executable / library easy. Instantiated only by the `impl_localize!` macro.
pub struct StaticParser<'a> {
    /// Bundles used for localization.
    /// Maps long-form locales (e.g. "en_US", not just "en") to their respective bundles.
    bundles: HashMap<&'static str, FluentBundle<'a>>,

    /// Available locales.
    available: Vec<&'static str>,

    /// Optimization: we always treat locales as &'static strs; this is used
    /// to convert &'a strs to &'static strs
    available_set: HashSet<&'static str>,

    /// The default locale chosen if no others can be determined.
    default_locale: &'static str,
}

impl<'a> StaticParser<'a> {
    /// Create a StaticParser.
    pub fn new(resources: &'a Resources, default_locale: &'static str) -> StaticParser<'a> {
        assert!(
            resources
                .0
                .iter()
                .find(|(locale, _)| *locale == default_locale)
                .is_some(),
            "default locale not available!"
        );

        let mut bundles = HashMap::new();
        let mut available = Vec::new();
        for (locale, resources) in resources.0.iter() {
            // confusingly, this value is used by fluent for number and date formatting only.
            // we have to implement looking up missing messages in other bundles ourselves.
            let fallback_chain = &[locale];

            let mut bundle = FluentBundle::new(fallback_chain);

            for resource in resources {
                bundle
                    .add_resource(resource)
                    .expect("failed to add resource");
            }
            bundles.insert(*locale, bundle);

            available.push(*locale);
        }
        available.sort();

        let available_set = available.iter().cloned().collect();

        StaticParser {
            bundles,
            available,
            available_set,
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
    ) -> Vec<&'static str> {
        let requested = accept_language.map(|accept_language| {
            let mut accepted = user_locales.to_owned();
            accepted.extend(&parse_accepted_languages(accept_language));
            accepted
        });
        let requested = match requested {
            Some(ref requested) => &requested[..],
            None => user_locales,
        };
        let result = negotiate_languages(
            requested,
            &self.available,
            Some(self.default_locale),
            &NegotiationStrategy::Filtering,
        );

        // prove to borrowck that all locales are static strings
        result
            .into_iter()
            .map(|l| {
                *self
                    .available_set
                    .get(l)
                    .expect("invariant violated: available and available_set have same contents")
            })
            .collect()
    }

    /// Localize a message.
    /// * `locale_chain`: a list of locales, in descending order of preference
    /// * `message`: a message ID
    /// * `args`: a slice of arguments to pass to Fluent.
    pub fn localize(
        &self,
        locale_chain: &[&'static str],
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
            let bundle = self
                .bundles
                .get(locale)
                .expect("invariant violated: available locales should have matching bundles");
            // this API is weirdly awful;
            // format returns Option<(String, Vec<FluentError>)>
            // which we have to cope with
            let result = bundle.format(message, args);

            if let Some((result, _errs)) = result {
                return Ok(result);

                // TODO: warn on errors here?
            }
        }
        // nowhere to fall back to
        Err(Error::NoTranslationsForMessage(format!(
            "no non-erroring translations for message {} in locale chain {:?}",
            message, locale_chain
        )))
    }

    pub fn has_message(&self, locale_chain: &[&'static str], message: &str) -> bool {
        locale_chain
            .iter()
            .flat_map(|locale| self.bundles.get(locale))
            .any(|bundle| bundle.has_message(message))
    }
}

/// Sources that have been parsed. Instantiated only by the `impl_localize!` macro.
///
/// This type is initialized in a lazy_static! in impl_localize!,
/// because FluentBundle can only take FluentResources by reference;
/// we have to store them somewhere to reference them.
/// This can go away once https://github.com/projectfluent/fluent-rs/issues/103 lands.
pub struct Resources(Vec<(&'static str, Vec<FluentResource>)>);

impl Resources {
    /// Parse a list of sources into a list of resources.
    pub fn new(sources: Sources) -> Resources {
        Resources(
            sources
                .iter()
                .map(|(locale, sources)| {
                    (
                        *locale,
                        sources
                            .iter()
                            .map(|source| {
                                FluentResource::try_new(source.to_string())
                                    .expect("baked .ftl translation failed to parse")
                            })
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}

/// Sources; an array mapping &'static strs to fluent source strings. Instantiated only by the `impl_localize!` macro.
pub type Sources = &'static [(&'static str, &'static [&'static str])];

pub use fluent_bundle::FluentValue as I18nValue;

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCES: Sources = &[
        (
            "en_US",
            &[r#"
greeting = Hello, { $name }! You are { $hours } hours old.
goodbye = Goodbye.
"#],
        ),
        (
            "en_AU",
            &[r#"
greeting = G'day, { $name }! You are { $hours } hours old.
goodbye = Hooroo.
"#],
        ),
        (
            "es_MX",
            &[r#"
greeting = ¡Hola, { $name }! Tienes { $hours } horas.
goodbye = Adiós.
"#],
        ),
        (
            "de_DE",
            &["greeting = Hallo { $name }! Du bist { $hours } Stunden alt."],
        ),
    ];

    #[test]
    fn basic() -> Result<()> {
        let resources = Resources::new(SOURCES);
        let bundles = StaticParser::new(&resources, "en_US");
        let name = FluentValue::from("Jamie");
        let hours = FluentValue::from(190321.31);
        let args = &[("name", &name), ("hours", &hours)][..];

        assert_eq!(
            bundles.localize(&["en_US"], "greeting", args)?,
            "Hello, Jamie! You are 190321.31 hours old."
        );
        assert_eq!(
            bundles.localize(&["es_MX"], "greeting", args)?,
            "¡Hola, Jamie! Tienes 190321.31 horas."
        );
        assert_eq!(
            bundles.localize(&["de_DE"], "greeting", args)?,
            "Hallo Jamie! Du bist 190321.31 Stunden alt."
        );

        // missing messages should fall back to first available
        assert_eq!(
            bundles.localize(&["de_DE", "es_MX", "en_US"], "goodbye", &[])?,
            "Adiós."
        );

        if let Ok(_) = bundles.localize(&["en_US"], "bananas", &[]) {
            panic!("Should return Err on missing message");
        }

        Ok(())
    }

    #[test]
    fn create_locale_chain() {
        let resources = Resources::new(SOURCES);
        let bundles = StaticParser::new(&resources, "en_US");

        // accept-language parser works + short-code lookup works
        assert_eq!(
            bundles.create_locale_chain(&[], Some("en_US, es_MX; q=0.5")),
            &["en_US", "en_AU", "es_MX"]
        );

        // first choice has precedence
        assert_eq!(
            bundles.create_locale_chain(&["es_MX"], Some("en_US; q=0.5")),
            &["es_MX", "en_US", "en_AU"]
        );

        // short codes work
        assert_eq!(
            bundles.create_locale_chain(&[], Some("en")),
            &["en_US", "en_AU"]
        );

        // default works
        assert_eq!(bundles.create_locale_chain(&[], None), &["en_US"]);

        // missing languages fall through to default
        assert_eq!(
            bundles.create_locale_chain(&["zh_HK"], Some("xy_ZW")),
            &["en_US"]
        );
    }
}
