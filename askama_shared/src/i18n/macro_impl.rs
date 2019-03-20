use accept_language::parse as accept_language_parse;
use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;

use super::super::{Error, Result};

pub use lazy_static::lazy_static;

pub type Sources = &'static [(Locale, &'static str)];

/// Parsed sources.
pub struct Resources(Vec<(Locale, FluentResource)>);

/// A known locale, with translations baked into the binary.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Locale(pub &'static str);

impl Resources {
    pub fn new(sources: Sources) -> Resources {
        let mut result = Resources(
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
        );
        result.0.sort_by_key(|r| r.0);
        result
    }
}

/// StaticParser is a type that handles accessing the translations baked into
/// the output executable / library easy.
/// Users should never need to interact with it; all uses are through the
/// `init_askama_i18n!()` macro or codegen for the `localize(...)` filter.
pub struct StaticParser<'a> {
    /// Bundles used for localization.
    bundles: HashMap<Locale, FluentBundle<'a>>,

    /// Available locales. Includes long form locales ("en_US" => [Locale("en_US")])
    /// and short-form locales ("en" => [Locale("en_US"), Locale("en-UK")]).
    locales: HashMap<&'static str, Vec<Locale>>,
    /// The default locale chosen if no others can be determined.
    default_locale: Locale,
}

impl<'a> StaticParser<'a> {
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

            println!("{:?}", locale);
        }

        StaticParser {
            bundles,
            locales,
            default_locale,
        }
    }

    /// Creates a locale chain; see the documentation of `new` on the `Localize` trait.
    ///
    pub fn create_locale_chain(
        &self,
        locale: Option<&str>,
        accept_language: Option<&str>,
    ) -> Vec<Locale> {
        let mut chain = vec![];

        let mut add = |locale_code: &str| {
            [
                &locale_code[..],  // e.g. "en-US"
                &locale_code[..2], // e.g. "en"
            ]
            .iter()
            .flat_map(|code| self.locales.get(code))
            .flat_map(|locales| locales.iter())
            .for_each(|result| {
                if !chain.contains(result) {
                    chain.push(*result);
                }
            });
        };

        locale.map(|locale| add(locale));
        accept_language.map(|accepts| {
            let accepts = accept_language_parse(accepts);
            for accept in accepts {
                add(&accept);
            }
        });

        if !chain.contains(&self.default_locale) {
            chain.push(self.default_locale);
        }

        chain
    }

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

        println!("{:#?}", bundles.locales);

        // accept-language parser works + short-code lookup works
        assert_eq!(
            bundles.create_locale_chain(None, Some("en_US, es_MX; q=0.5")),
            &[Locale("en_US"), Locale("en_AU"), Locale("es_MX")]
        );

        // first choice has precedence
        assert_eq!(
            bundles.create_locale_chain(Some("es_MX"), Some("en_US; q=0.5")),
            &[Locale("es_MX"), Locale("en_US"), Locale("en_AU")]
        );

        // short codes work
        assert_eq!(
            bundles.create_locale_chain(None, Some("en")),
            &[Locale("en_AU"), Locale("en_US")]
        );

        // default works
        assert_eq!(bundles.create_locale_chain(None, None), &[Locale("en_US")]);

        // missing languages fall through to default
        assert_eq!(
            bundles.create_locale_chain(Some("zh_HK"), Some("xy_ZW")),
            &[Locale("en_US")]
        );
    }
}
