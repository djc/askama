//! Module `i18n` provides tools used by askama's internationalization /
//! localization system, which you can use to translate your templates into other languages.

use fluent_bundle::{FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;

use super::{Error, Result};

pub type _Sources = &'static [(&'static str, &'static str)];
pub type _Resources = HashMap<&'static str, FluentResource>;
pub type _FallbackChains = &'static [&'static [&'static str]];

pub fn _parse_all(sources: Sources) -> Resources {
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
pub struct _FluentBundles<'a> {
    bundles: HashMap<&'static str, FluentBundle<'a>>,
}

impl<'a> _FluentBundles<'a> {
    pub fn new(resources: &'a _Resources, fallback_chains: _FallbackChains) -> _FluentBundles<'a> {
        let mut bundles = HashMap::new();
        for (locale, resource) in resources.iter() {
            let locale: &'static str = *locale;

            let chain = fallback_chains
                .iter()
                .find(|chain| chain[0] == locale)
                .unwrap_or(&&[locale][..]);

            let mut bundle = FluentBundle::new(chain);

            bundle.add_resource(resource);
            bundles.insert(locale, bundle);
        }

        _FluentBundles { bundles }
    }

    pub fn localize(
        &self,
        locale: &str,
        path: &str,
        args: Option<&HashMap<&str, FluentValue>>,
    ) -> Result<String> {
        let bundle = self.bundles.get(locale).unwrap_or_else(|| {
            // TODO: use fallback chains here? might be confusing, could just error
            &self.bundles["en-us"]
        });
        let args_map = if args.len() > 0 {
            Some(
                args.into_iter()
                    .map(|&(a, b)| (a, b))
                    .collect::<HashMap<_, _>>(),
            )
        } else {
            None
        };
        // this API is weirdly awful;
        // format returns Option<(String, Vec<FluentError>)>
        // which we have to cope with
        let result = bundle.format(path, args_map.as_ref());

        if let Some((result, mut errs)) = result {
            if errs.len() > 0 {
                // TODO handle more than 1 error
                Err(Error::I18n(Some(errs.pop().unwrap())))
            } else {
                Ok(result)
            }
        } else {
            Err(Error::I18n(None))
        }
    }
}

#[cfg(test)]
mod test {}
