//! Internationalization codegen.

use proc_macro2::TokenStream;

use fluent_syntax::ast;
use quote::quote;
use std::collections::BTreeMap;
use std::env;
use std::fs::{read_to_string, DirEntry};
use std::path::{Path, PathBuf};

use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{bracketed, parenthesized, token, Ident, LitStr, Token};

pub fn impl_localize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(item as ImplLocalize);

    let mut root =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").expect("askama doesn't work without cargo"));
    root.push(&ast.path);

    assert!(root.is_dir(), "no such directory: {:?}", root);

    let mut sources = vec![];

    let mut includes = vec![];

    let mut message_counts = BTreeMap::new();

    let mut had_errors = false;

    for child in children(&root) {
        let locale = child.file_name().to_string_lossy().to_string();

        message_counts.insert(locale.clone(), 0);

        if !child.file_type().unwrap().is_dir() {
            continue;
        }

        let mut message_count = 0;

        let mut source = String::new();

        for ftl_file in children(&child.path()) {
            if ftl_file
                .path()
                .extension()
                .map(|x| x != "ftl")
                .unwrap_or(false)
            {
                // not an ftl file
                continue;
            }
            let file_source = read_to_string(&ftl_file.path()).expect("failed to read .ftl file");
            let path = ftl_file.path();
            let pretty_path = path
                .strip_prefix(&root)
                .expect("failed to strip prefix of .ftl path")
                .display();

            let parse_check = fluent_bundle::FluentResource::try_new(file_source.clone())
                .unwrap_or_else(|(res, errs)| {
                    eprintln!(
                        "askama i18n error: fluent parse errors in `{}`",
                        pretty_path
                    );
                    for err in errs {
                        let (line, col) = linecol(&file_source, err.pos.0);
                        eprintln!(
                            "askama i18n error:     {}:{}:{}: {:?}",
                            pretty_path, line, col, err.kind
                        );
                    }
                    had_errors = true;
                    res
                });

            for entry in &parse_check.ast().body {
                if let &ast::ResourceEntry::Entry(ast::Entry::Message(ref m)) = entry {
                    message_count += 1 + m.attributes.len();
                }
            }

            includes.push(ftl_file.path().display().to_string());

            source.push_str(&file_source);

            message_counts.insert(locale.clone(), message_count);
        }

        if source.len() == 0 {
            // empty directory
            continue;
        }

        sources.push((locale, source));
    }

    if had_errors {
        panic!("askama i18n error: fluent source files have errors, not continuing")
    }
    if includes.len() == 0 {
        eprintln!("askama i18n warning: no fluent .ftl translation files provided in i18n directory, localize() won't do much");
    }

    let name = ast.name;
    let default_locale = ast.default_locale;

    if sources
        .iter()
        .filter(|(locale, _)| locale == &default_locale)
        .next()
        .is_none()
    {
        panic!("askama i18n error: no code for default locale");
    }

    let coverage = coverage(message_counts);
    let sources = sources
        .into_iter()
        .map(|(locale, source)| quote! { (#locale, #source) })
        .collect::<Vec<_>>();
    let includes = includes
        .into_iter()
        .map(|i| quote! { include_bytes!(#i); })
        .collect::<Vec<_>>();

    let result = (quote! {
        /// Internationalization support. Automatically generated from files in the `i18n` folder.
        pub struct #name(&'static str);

        impl ::askama::Localize for #name {

            fn new(locale: Option<&str>, accept_language: Option<&str>) -> Self {
                #name(__i18n_hidden::STATIC_PARSER.choose_locale(locale, accept_language))
            }

            #[inline]
            fn localize(&self,
                message: &str,
                args: &[(&str, &askama::i18n::I18nValue)])
                    -> ::askama::Result<String> {
                    __i18n_hidden::STATIC_PARSER.localize(self.0, message, args)
            }
        }

        #[doc(hidden)]
        mod __i18n_hidden {
            use ::askama::i18n::I18nValue;
            use ::askama::i18n::macro_impl::{
                StaticParser, Resources,
                Sources, lazy_static,
            };
            pub const SOURCES: Sources = &[
                #(#sources),*
            ];

            lazy_static! {
                static ref RESOURCES: Resources = Resources::new(SOURCES);
                pub static ref STATIC_PARSER: StaticParser<'static> =
                    StaticParser::new(&RESOURCES, #default_locale);
            }

            #[allow(unused)]
            fn i_depend_on_these_files() {
                #(#includes)*
            }

            #[cfg(test)]
            mod tests {
                #[test]
                fn parse() {
                    let _parse_all_sources = &*super::STATIC_PARSER;
                }

                #[test]
                fn i18n_coverage() {
                    #coverage
                }
            }
        }
    })
    .into();
    result
}

/// Generate a test that gives a fluent coverage report when run.
fn coverage(message_counts: BTreeMap<String, usize>) -> TokenStream {
    if message_counts.len() == 0 {
        return quote! {
            eprintln!("askama-i18n-coverage: no translation files provided, coverage vacuous");
        };
    }

    let max = message_counts.values().max().unwrap();

    let coverages = message_counts
        .iter()
        .map(|(locale, message_count)| {
            let percent = 100.0 * (*message_count as f32) / (*max as f32);
            let message = format!(
                "askama-i18n-coverage: {} \t{:3.0}% ({}/{})",
                locale, percent, message_count, max
            );
            quote! {
                eprintln!(#message);
            }
        })
        .collect::<Vec<_>>();

    let found_bad = message_counts.values().any(|count| count < max);
    let end = if found_bad {
        quote! {
        eprintln!("askama-i18n-coverage: help: to get accurate results, make sure \
        that messages\n not used directly by your software are prefixed with a dash (`-`).");
        }
    } else {
        quote! {
            eprintln!("askama-i18n-coverage: fully covered, nice job :)");
        }
    };

    quote! {
        eprintln!("askama-i18n-coverage: translated messages and attributes per locale:");
        #(#coverages)*
        #end
    }
}

fn children(path: &Path) -> impl Iterator<Item = DirEntry> {
    path.read_dir()
        .expect("no such path")
        .map(|entry| entry.expect("stop changing the filesystem underneath me"))
}

fn linecol(src: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in src.chars().enumerate() {
        if i == offset {
            return (line, col);
        }

        col += 1;
        if c == '\n' {
            col = 0;
            line += 1;
        }
    }

    (line, col)
}

struct NamedArg {
    name: Ident,
    value: String,
}

impl Parse for NamedArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<LitStr>()?.value();
        Ok(NamedArg { name, value })
    }
}

struct ImplLocalize {
    name: Ident,
    path: String,
    default_locale: String,
}

impl Parse for ImplLocalize {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut path = None;
        let mut default_locale = None;

        input.parse::<Token![#]>()?;
        let annotation;
        bracketed!(annotation in input);

        let ann_name = annotation.parse::<Ident>()?;
        if ann_name.to_string() != "localize" {
            return Err(syn::parse::Error::new(
                ann_name.span(),
                "expected `#[localize]` or `#[localize(path = \"...\", default_locale = \"...\")]",
            ));
        }

        let lookahead = annotation.lookahead1();
        if lookahead.peek(token::Paren) {
            let args;
            parenthesized!(args in annotation);
            let args = Punctuated::<NamedArg, Token![,]>::parse_terminated(&args)?;
            for arg in args.iter() {
                match &arg.name.to_string()[..] {
                    "path" => path = Some(arg.value.clone()),
                    "default_locale" => default_locale = Some(arg.value.clone()),
                    _ => {
                        return Err(syn::parse::Error::new(
                            arg.name.span(),
                            "expected one of `path = \"...\"`, `default_locale = \"...\"`",
                        ));
                    }
                }
            }
        }

        let path = path.unwrap_or("i18n".to_string());
        let default_locale = default_locale.unwrap_or("en_US".to_string());

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![pub]) {
            // note: currently the output is always pub. Shrug emoji
            input.parse::<Token![pub]>()?;
        }

        input.parse::<Token![struct]>()?;
        let name = input.parse::<Ident>()?;
        let dummy;
        parenthesized!(dummy in input);
        dummy.parse::<Token![_]>()?;
        input.parse::<Token![;]>()?;

        Ok(ImplLocalize {
            name,
            path,
            default_locale,
        })
    }
}
