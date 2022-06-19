use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::{DirEntry, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use fluent_syntax::ast::{
    Expression, InlineExpression, PatternElement, Resource, Variant, VariantKey,
};
use fluent_syntax::parser::parse_runtime;
use fluent_templates::lazy_static::lazy_static;
use fluent_templates::loader::build_fallbacks;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote_spanned;
use serde::Deserialize;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse2, Visibility};
use toml::from_str;
use unic_langid::LanguageIdentifier;

use crate::CompileError;

macro_rules! mk_static {
    ($(let $ident:ident: $ty:ty = $expr:expr;)*) => {
        $(
            let $ident = {
                let value: Option<$ty> = Some($expr);
                unsafe {
                    static mut VALUE: Option<$ty> = None;
                    VALUE = value;
                    match &VALUE {
                        Some(value) => value,
                        None => unreachable!(),
                    }
                }
            };
        )*
    };
}

struct Variable {
    vis: Visibility,
    name: Ident,
}

impl Parse for Variable {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let vis = input.parse().unwrap_or(Visibility::Inherited);
        let name = input.parse()?;
        Ok(Variable { vis, name })
    }
}

struct Configuration {
    pub(crate) fallback: LanguageIdentifier,
    pub(crate) use_isolating: bool,
    pub(crate) core_locales: Option<(PathBuf, Resource<String>)>,
    pub(crate) locales: Vec<(LanguageIdentifier, Vec<(PathBuf, Resource<String>)>)>,
    pub(crate) fallbacks: &'static HashMap<LanguageIdentifier, Vec<LanguageIdentifier>>,
    pub(crate) assets_dir: PathBuf,
}

#[derive(Default, Deserialize)]
struct I18nConfig {
    #[serde(default)]
    pub(crate) fallback_language: Option<String>,
    #[serde(default)]
    pub(crate) fluent: Option<I18nFluent>,
}

#[derive(Default, Deserialize)]
struct I18nFluent {
    #[serde(default)]
    pub(crate) assets_dir: Option<PathBuf>,
    #[serde(default)]
    pub(crate) core_locales: Option<PathBuf>,
    #[serde(default)]
    pub(crate) use_isolating: Option<bool>,
}

fn format_err(path: &Path, err: impl Display) -> String {
    format!("error processing {:?}: {}", path, err)
}

fn read_resource(path: PathBuf) -> Result<(PathBuf, Resource<String>), String> {
    let mut buf = String::new();
    OpenOptions::new()
        .read(true)
        .open(&path)
        .map_err(|err| format_err(&path, err))?
        .read_to_string(&mut buf)
        .map_err(|err| format_err(&path, err))?;

    let resource = match parse_runtime(buf) {
        Ok(resource) => resource,
        Err((_, err_vec)) => return Err(format_err(&path, err_vec.first().unwrap())),
    };
    Ok((path, resource))
}

fn read_lang_dir(
    entry: Result<DirEntry, std::io::Error>,
) -> Result<Option<(LanguageIdentifier, Vec<(PathBuf, Resource<String>)>)>, String> {
    let entry = match entry {
        Ok(entry) => entry,
        Err(_) => return Ok(None),
    };

    let language = entry
        .file_name()
        .to_str()
        .and_then(|s| LanguageIdentifier::from_str(s).ok());
    let language: LanguageIdentifier = match language {
        Some(language) => language,
        None => return Ok(None),
    };

    let dir_iter = match entry.path().read_dir() {
        Ok(dir_iter) => dir_iter,
        Err(_) => return Ok(None),
    };
    let mut resources = vec![];
    for entry in dir_iter {
        if let Ok(entry) = entry {
            let path = entry.path();
            if entry
                .path()
                .to_str()
                .map(|s| s.ends_with(".ftl"))
                .unwrap_or(false)
            {
                resources.push(read_resource(path)?);
            };
        }
    }
    if resources.is_empty() {
        return Ok(None);
    }

    resources.sort_by(|(l, _), (r, _)| Path::cmp(l, r));
    Ok(Some((language, resources)))
}

fn read_configuration() -> Result<Configuration, String> {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = root.canonicalize().unwrap_or(root);

    let i18n_toml = root.join("i18n.toml");
    let config = match i18n_toml.exists() {
        false => I18nConfig::default(),
        true => {
            let mut buf = String::new();
            OpenOptions::new()
                .read(true)
                .open(&i18n_toml)
                .map_err(|err| format_err(&i18n_toml, err))?
                .read_to_string(&mut buf)
                .map_err(|err| format_err(&i18n_toml, err))?;
            from_str(&buf).map_err(|err| format_err(&i18n_toml, err))?
        }
    };
    let fluent = config.fluent.unwrap_or_default();

    let fallback = config.fallback_language.as_deref().unwrap_or("en");
    let fallback: LanguageIdentifier = match fallback.parse() {
        Ok(fallback) => fallback,
        Err(err) => {
            return Err(format!(
                "not a valid LanguageIdentifier {:?} for fallback_language: {}",
                err, fallback,
            )
            .into())
        }
    };

    let core_locales = match fluent.core_locales {
        Some(path) => {
            let path = match path.is_absolute() {
                true => path,
                false => root.join(path),
            };
            if path.to_str().is_none() {
                return Err(format!(
                    "core_locales path contains illegal UTF-8 characters: {:?}",
                    path,
                ));
            };
            Some(read_resource(path)?)
        }
        None => None,
    };

    let assets_dir = match fluent.assets_dir {
        Some(path) if path.is_absolute() => todo!(),
        Some(path) => root.join(&path),
        None => root.join("i18n"),
    };
    let mut locales = {
        let mut locales = vec![];
        for entry in assets_dir
            .read_dir()
            .map_err(|err| format_err(&assets_dir, err))?
        {
            if let Some(datum) = read_lang_dir(entry)? {
                locales.push(datum);
            }
        }
        locales
    };
    locales.sort_by(|(l1, _), (l2, _)| LanguageIdentifier::cmp(l1, l2));

    mk_static! {
        let locales_: Vec<LanguageIdentifier> = locales.iter().map(|(l, _)| l.clone()).collect();
        let fallbacks: HashMap<LanguageIdentifier, Vec<LanguageIdentifier>> = build_fallbacks(
            &locales_,
        );
    };

    Ok(Configuration {
        fallback,
        use_isolating: fluent.use_isolating.unwrap_or(false),
        core_locales,
        locales,
        fallbacks,
        assets_dir,
    })
}

fn get_i18n_config() -> Result<&'static Configuration, CompileError> {
    lazy_static! {
        static ref CONFIGURATION: Result<Configuration, String> = read_configuration();
    }
    match &*CONFIGURATION {
        Ok(configuration) => Ok(configuration),
        Err(err) => Err(err.as_str().into()),
    }
}

pub(crate) fn derive(input: TokenStream) -> Result<TokenStream, CompileError> {
    let configuration = get_i18n_config()?;

    let input: TokenStream2 = input.into();
    let span = input.span();
    let variable: Variable = match parse2(input) {
        Ok(variable) => variable,
        Err(err) => return Err(format!("could not parse localize!(…): {}", err).into()),
    };

    let vis = variable.vis;
    let name = variable.name;
    let assets_dir = configuration.assets_dir.to_str().unwrap();
    let fallback = configuration.fallback.to_string();
    let core_locales = configuration.core_locales.as_ref().map(|(s, _)| {
        let s = s.to_str().unwrap();
        quote_spanned!(span => core_locales: #s,)
    });
    let customise = match configuration.use_isolating {
        false => Some(quote_spanned!(span => customise: |b| b.set_use_isolating(false),)),
        true => None,
    };

    let ts = quote_spanned! {
        span =>
        #vis static #name:
            ::askama::fluent_templates::once_cell::sync::Lazy::<
                ::askama::fluent_templates::StaticLoader
            > = ::askama::fluent_templates::once_cell::sync::Lazy::new(|| {
                mod fluent_templates {
                    // RATIONALE: the user might not use fluent_templates directly.
                    pub use ::askama::fluent_templates::*;
                    pub mod once_cell {
                        pub mod sync {
                            pub use ::askama::Unlazy as Lazy;
                        }
                    }
                }
                ::askama::fluent_templates::static_loader! {
                    pub static LOCALES = {
                        locales: #assets_dir,
                        fallback_language: #fallback,
                        #core_locales
                        #customise
                    };
                }
                LOCALES.take()
            });
    };
    Ok(ts.into())
}

pub(crate) fn arguments_of(text_id: &str) -> Result<HashSet<&'static str>, CompileError> {
    let config = get_i18n_config()?;
    let entry = config.fallbacks[&config.fallback]
        .iter()
        .filter_map(|l1| {
            config
                .locales
                .binary_search_by(|(l2, _)| LanguageIdentifier::cmp(l2, l1))
                .ok()
        })
        .flat_map(|index| &config.locales[index].1)
        .chain(config.core_locales.iter())
        .flat_map(|(_, resource)| &resource.body)
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(entry) => Some(entry),
            _ => None,
        })
        .find_map(|entry| match entry.id.name == text_id {
            true => Some(entry),
            false => None,
        })
        .ok_or_else(|| CompileError::from(format!("text_id {:?} not found", text_id)))?;

    let keys = entry
        .value
        .iter()
        .flat_map(|v| v.elements.iter())
        .filter_map(|p| match p {
            PatternElement::Placeable { expression } => Some(expression),
            _ => None,
        })
        .flat_map(expr_to_key)
        .collect();
    Ok(keys)
}

fn expr_to_key(expr: &'static Expression<String>) -> Vec<&'static str> {
    let (selector, variants): (&InlineExpression<String>, &[Variant<String>]) = match expr {
        Expression::Select { selector, variants } => (selector, variants),
        Expression::Inline(selector) => (selector, &[]),
    };

    let variant_keys = variants.iter().filter_map(|v| match &v.key {
        VariantKey::Identifier { name } => Some(name.as_str()),
        _ => None,
    });

    let variant_values = variants
        .iter()
        .flat_map(|v| v.value.elements.iter())
        .filter_map(|v| match v {
            PatternElement::Placeable { expression } => Some(expression),
            _ => None,
        })
        .flat_map(expr_to_key);

    let selector_keys = inline_expr_to_key(selector);

    let mut v = vec![];
    v.extend(variant_keys);
    v.extend(variant_values);
    v.extend(selector_keys);
    v
}

fn inline_expr_to_key(selector: &'static InlineExpression<String>) -> Vec<&'static str> {
    let mut v = vec![];
    v.extend(selector_placeable(selector));
    v.extend(selector_variable(selector));
    v.extend(selector_function(selector));
    v
}

fn selector_placeable(e: &'static InlineExpression<String>) -> impl Iterator<Item = &'static str> {
    let e = match e {
        InlineExpression::Placeable { expression } => Some(expression),
        _ => None,
    };
    e.into_iter().flat_map(|e| expr_to_key(e))
}

fn selector_variable(e: &'static InlineExpression<String>) -> impl Iterator<Item = &'static str> {
    let id = match e {
        InlineExpression::VariableReference { id } => Some(id.name.as_str()),
        _ => None,
    };
    id.into_iter()
}

fn selector_function(e: &'static InlineExpression<String>) -> impl Iterator<Item = &'static str> {
    let arguments = match e {
        InlineExpression::FunctionReference { arguments, .. } => Some(arguments),
        _ => None,
    };
    arguments.into_iter().flat_map(|a| {
        a.named
            .iter()
            .map(|n| &n.value)
            .chain(&a.positional)
            .flat_map(inline_expr_to_key)
    })
}
