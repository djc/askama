//! Internationalization codegen.
use proc_macro::TokenStream;

use std::env;
use std::fmt::Write;
use std::fs::{read_to_string, DirEntry};
use std::path::{Path, PathBuf};

pub fn init_askama_i18n(item: TokenStream) -> TokenStream {
    // TODO fancier parsing
    let path = syn::parse_macro_input!(item as syn::LitStr).value();

    let mut root = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    root.push(path);

    assert!(root.is_dir(), "no such directory: {:?}", root);

    let mut sources = "&[\n".to_string();

    for child in children(&root) {
        let locale = child.file_name().to_string_lossy().to_string();

        // TODO: is this a valid locale-string check?
        if locale.len() != 5 || locale.chars().nth(2).unwrap() != '-' {
            continue;
        }

        let mut source = String::new();

        for ftl_file in children(&child.path()) {
            source.push_str(&read_to_string(&ftl_file.path()).expect("failed to read"));
        }

        writeln!(
            sources,
            r#####"    ("{}", r####"{}"####),   "#####,
            locale, source
        )
        .unwrap();
    }

    write!(sources, "\n]").unwrap();

    let result = template(&sources, r#"&[&["en-US"]]"#);
    //println!("{}", result);
    result.parse().unwrap()
}

fn children(path: &Path) -> impl Iterator<Item = DirEntry> {
    path.read_dir()
        .expect("no such path")
        .map(|entry| entry.expect("stop changing the filesystem underneath me"))
}

fn template(sources: &str, fallback_chains: &str) -> String {
    format!(
        r##"
// Internationalization support. Automatically generated from files in the `i18n` folder.

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

#[cfg(test)]
mod tests {{
    #[test]
    fn parse() {{
        let _parse_all_sources = &*super::LOCALIZATIONS;
    }}
}}
"##,
        fallback_chains = fallback_chains,
        sources = sources
    )
}
