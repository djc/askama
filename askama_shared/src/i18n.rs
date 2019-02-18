//! Module `i18n` provides tools used by askama's internationalization /
//! localization system, which you can use to translate your templates into other languages.

use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;

use super::{Error, Result};

pub type Sources = &'static [(&'static str, &'static str)];
pub type Resources = HashMap<&'static str, FluentResource>;
pub type FallbackChains = &'static [&'static [&'static str]];

pub fn parse_all(sources: Sources) -> Resources {
    sources
        .into_iter()
        .map(|(locale, source)| {
            (
                *locale,
                FluentResource::try_new(source.to_string())
                    .expect("baked .ftl translation failed to parse"),
            )
        })
        .collect()
}
/// FluentBundles is a type that handles accessing the translations baked into
/// the output executable / library easy.
/// Users should never need to interact with it; all uses are through the
/// `init_askama_i18n!()` macro or codegen for the `localize(...)` filter.
pub struct FluentBundles<'a> {
    bundles: HashMap<&'static str, FluentBundle<'a>>,
}

impl<'a> FluentBundles<'a> {
    pub fn new(resources: &'a Resources, fallback_chains: FallbackChains) -> FluentBundles<'a> {
        let mut bundles = HashMap::new();
        for (locale, resource) in resources.iter() {
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
        }

        FluentBundles { bundles }
    }

    pub fn localize(
        &self,
        locale: &str,
        path: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> Result<String> {
        let bundle = self.bundles.get(locale).unwrap_or_else(|| {
            // TODO: use fallback chains here? might be confusing, could just error
            &self.bundles["en-US"]
        });
        // this API is weirdly awful;
        // format returns Option<(String, Vec<FluentError>)>
        // which we have to cope with
        let result = bundle.format(path, args);

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
            Err(Error::I18n(None))
        }
    }
}

#[cfg(test)]
mod test {
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
        let resources = parse_all(SOURCES);
        let bundles = FluentBundles::new(&resources, FALLBACK_CHAINS);
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
