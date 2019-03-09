use accept_language::parse as accept_language_parse;
use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;

use super::super::{Error, Result};

pub use lazy_static::lazy_static;

pub type Sources = &'static [(&'static str, &'static str)];

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
    /// Bundles used for localization.
    bundles: HashMap<&'static str, FluentBundle<'a>>,
    /// Available locales. Includes long form locales ("en_US" => "en_US")
    /// and short-form locales ("en" => "en_US").
    locales: HashMap<&'static str, &'static str>,
    /// The default locale chosen if no others can be determined.
    default_locale: &'static str,
}

impl<'a> StaticParser<'a> {
    pub fn new(
        resources: &'a Resources,
        default_locale: &'static str,
    ) -> StaticParser<'a> {
        assert!(
            resources.0.contains_key(default_locale),
            "default locale not in available languages!"
        );

        let mut bundles = HashMap::new();
        let mut locales = HashMap::new();
        for (locale, resource) in resources.0.iter() {
            let fallback_chain = &[*locale, &locale[..2], default_locale, &default_locale[..2]];

            let mut bundle = FluentBundle::new(fallback_chain);

            bundle
                .add_resource(resource)
                .expect("failed to add resource");
            bundles.insert(*locale, bundle);
            locales.insert(*locale, *locale);

            // TODO: allow customizing which locale should be chosen for a short-form locale
            // (e.g. should "en" map to "en_US" or "en_UK"?)
            let short = &locale[..2];
            locales.entry(short).or_insert(*locale);
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
        accept_language: Option<&str>,
    ) -> &'static str {
        if let Some(locale) = locale {
            if let Some(&static_locale) = self.locales.get(locale) {
                return static_locale;
            } else {
                // TODO: warn here?
            }
        }
        if let Some(accepts) = accept_language {
            // ordered list of language strings
            let accepts = accept_language_parse(accepts);

            for language in accepts.iter() {
                if let Some(static_locale) = self.locales.get(&language[..]) {
                    return static_locale;
                }
            }
        }
        self.default_locale
    }

    pub fn localize(
        &self,
        locale: &str,
        message: &str,
        args: &[(&str, &FluentValue)],
    ) -> Result<String> {
        let bundle = self.bundles.get(locale).unwrap_or_else(|| 
            // TODO: it would be possible to do something more elaborate here
            &self.bundles[self.default_locale]
        );

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
            "en_US",
            "greeting = Hello, { $name }! You are { $hours } hours old.",
        ),
        (
            "es_MX",
            "greeting = ¡Hola, { $name }! Tienes { $hours } horas.",
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
            bundles.localize("en_US", "greeting", args)?,
            "Hello, Jamie! You are 190321.31 hours old."
        );
        assert_eq!(
            bundles.localize("es_MX", "greeting", args)?,
            "¡Hola, Jamie! Tienes 190321.31 horas."
        );

        // missing locales should use english
        assert_eq!(
            bundles.localize("zh_HK", "greeting", args)?,
            "Hello, Jamie! You are 190321.31 hours old."
        );

        if let Ok(_) = bundles.localize("en_US", "bananas", &[]) {
            panic!("Should return Err on missing message");
        }

        Ok(())
    }

    #[test]
    fn choose_locale() {
        let resources = Resources::new(SOURCES);
        let bundles = StaticParser::new(&resources, "en_US");

        // first choice has precedence
        assert_eq!(
            bundles.choose_locale(Some("es_MX"), Some("en_US; q=0.5")),
            "es_MX"
        );
        // accept-language parser works
        assert_eq!(bundles.choose_locale(None, Some("es_MX; q=0.5")), "es_MX");
        // default works
        assert_eq!(bundles.choose_locale(None, None), "en_US");
        // missing languages fall through to default
        assert_eq!(
            bundles.choose_locale(Some("zh_HK"), Some("de_DE, en_NZ")),
            "en_US"
        );
    }
}