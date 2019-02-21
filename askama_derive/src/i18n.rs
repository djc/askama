//! Internationalization codegen.
use proc_macro::TokenStream;

use fluent_syntax::ast;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Write;
use std::fs::{read_to_string, DirEntry};
use std::path::{Path, PathBuf};

pub fn init_askama_i18n(item: TokenStream) -> TokenStream {
    // TODO fancier parsing
    let path = syn::parse_macro_input!(item as syn::LitStr).value();

    let mut root =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").expect("askama doesn't work without cargo"));
    root.push(path);

    assert!(root.is_dir(), "no such directory: {:?}", root);

    let mut sources = "&[\n".to_string();

    // TODO: we could just use these as the sources and parse separately,
    // might avoid contamination... also this might include sources twice
    // if the linker doesn't throw out the useless strings
    let mut includes = "".to_string();

    let mut message_counts = BTreeMap::new();

    let mut had_errors = false;

    for child in children(&root) {
        let locale = child.file_name().to_string_lossy().to_string();

        message_counts.insert(locale.clone(), 0);

        // TODO: is this a valid locale-string check?
        if locale.len() != 5 || locale.chars().nth(2).unwrap() != '-' {
            continue;
        }

        let mut message_count = 0;

        let mut source = String::new();

        for ftl_file in children(&child.path()) {
            let file_source = read_to_string(&ftl_file.path()).expect("failed to read .ftl file");
            let path = ftl_file.path();
            let pretty_path = path
                .strip_prefix(&root)
                .expect("failed to strip prefix of .ftl path")
                .display();

            let parse_check = fluent_bundle::FluentResource::try_new(file_source.clone())
                .unwrap_or_else(|(res, errs)| {
                    // TODO: how should this be formatted?
                    println!("error: fluent parse errors in `{}`", pretty_path);
                    for err in errs {
                        let (line, col) = linecol(&file_source, err.pos.0);
                        println!(
                            "error:     {}:{}:{}: {:?}",
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

            writeln!(
                includes,
                "    include_bytes!(\"{}\");",
                ftl_file.path().display()
            )
            .unwrap();

            source.push_str(&file_source);

            message_counts.insert(locale.clone(), message_count);
        }

        writeln!(
            sources,
            r#####"    ("{}", r####"{}"####),   "#####,
            locale, source
        )
        .unwrap();
    }

    if had_errors {
        // TODO: compile-fail test?
        panic!("fluent source files have errors, not continuing")
    }
    if includes.len() == 0 {
        eprintln!("warning: no fluent .ftl translation files provided in i18n directory, localize() won't do much");
    }

    write!(sources, "\n]").unwrap();

    let result = format!(
        r##"// Internationalization support. Automatically generated from files in the `i18n` folder.

use ::askama::i18n::{{
    Localizations, FluentResource, FluentValue, Resources,
    Sources, FallbackChains, parse_all, lazy_static,
}};
const SOURCES: Sources = {sources};

const FALLBACK_CHAINS: FallbackChains = {fallback_chains};

lazy_static! {{
    static ref RESOURCES: Resources = parse_all(SOURCES);
    pub static ref LOCALIZATIONS: Localizations<'static> = 
        Localizations::new(&RESOURCES, FALLBACK_CHAINS);
}}

fn _bs() {{
    {includes}
}}

#[cfg(test)]
mod tests {{
    #[test]
    fn parse() {{
        let _parse_all_sources = &*super::LOCALIZATIONS;
    }}

    #[test]
    fn i18n_coverage() {{
        {coverage}
    }}
}}
"##,
        fallback_chains = r#"&[&["en-US"]]"#,
        sources = sources,
        includes = includes,
        coverage = coverage(message_counts)
    );

    result.parse().unwrap()
}

/// Generate a test that gives a fluent coverage report when run.
fn coverage(message_counts: BTreeMap<String, usize>) -> String {
    let mut result =
        "eprintln!(\"askama-i18n-coverage: translated messages and attributes per locale:\");"
            .to_string();
    let max = message_counts.values().max();
    if max.is_none() {
        writeln!(
            result,
            "eprintln!(\"askama-i18n-coverage: no translation files provided, coverage vacuous\");"
        )
        .unwrap();
        return result;
    }
    let max = max.unwrap();
    let mut found_bad = false;

    for (locale, message_count) in message_counts.iter() {
        if message_count < max {
            found_bad = true;
        }
        let percent = 100.0 * (*message_count as f32) / (*max as f32);
        writeln!(
            result,
            r#"eprintln!("askama-i18n-coverage: {} \t{:3.0}% ({}/{})");"#,
            locale, percent, message_count, max
        )
        .unwrap();
    }

    if found_bad {
        writeln!(result, "eprintln!(\"askama-i18n-coverage: help: to get accurate results, make sure \
            that messages\n not used directly by your software are prefixed with an underscore (`_`).\");").unwrap();
    } else {
        writeln!(
            result,
            "eprintln!(\"askama-i18n-coverage: fully covered, nice job :)\");"
        )
        .unwrap();
    }

    result
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
