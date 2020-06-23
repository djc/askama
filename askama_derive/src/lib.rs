extern crate proc_macro;

use askama_shared::heritage::{Context, Heritage};
use askama_shared::input::{Print, Source, TemplateInput};
use askama_shared::parser::{parse, Expr, Node};
use askama_shared::{generator, get_template_source, read_config_file, Config, Integrations};
use proc_macro::TokenStream;

use std::collections::HashMap;
use std::path::PathBuf;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Check that an attribute called `template()` exists and that it is
    // the proper type (list).
    let meta = ast
        .attrs
        .iter()
        .find_map(|attr| match attr.parse_meta() {
            Ok(m) => {
                if m.path().is_ident("template") {
                    Some(m)
                } else {
                    None
                }
            }
            Err(e) => panic!("unable to parse attribute: {}", e),
        })
        .expect("no attribute 'template' found");

    let meta_list = match meta {
        syn::Meta::List(inner) => inner,
        _ => panic!("attribute 'template' has incorrect type"),
    };

    let item = syn::Item::from(ast);
    let config_toml = read_config_file();
    let config = Config::new(&config_toml);
    let input = TemplateInput::new(&meta_list, &item, &config);
    build_template(input).parse().unwrap()
}

#[proc_macro_attribute]
pub fn template(meta: TokenStream, item: TokenStream) -> TokenStream {
    let item = match syn::parse::<syn::Item>(item) {
        syn::Item::Struct(item) => item,
        _ => panic!("only struct items are supported for now"),
    };

    let config_toml = read_config_file();
    let config = Config::new(&config_toml);
    let meta = syn::parse::<syn::MetaList>(meta);
    let input = TemplateInput::new(&meta, &item, &config);
    build_template(input).parse().unwrap()
}

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
fn build_template(input: TemplateInput) -> String {
    let source: String = match input.source {
        Source::Source(ref s) => s.clone(),
        Source::Path(_) => get_template_source(&input.path),
    };

    let mut sources = HashMap::new();
    find_used_templates(&input, &mut sources, source);

    let mut parsed = HashMap::new();
    for (path, src) in &sources {
        parsed.insert(path, parse(src, input.syntax));
    }

    let mut contexts = HashMap::new();
    for (path, nodes) in &parsed {
        contexts.insert(*path, Context::new(&input.config, path, nodes));
    }

    let ctx = &contexts[&input.path];
    let heritage = if !ctx.blocks.is_empty() || ctx.extends.is_some() {
        Some(Heritage::new(ctx, &contexts))
    } else {
        None
    };

    if input.print == Print::Ast || input.print == Print::All {
        eprintln!("{:?}", parsed[&input.path]);
    }

    let code = generator::generate(&input, &contexts, &heritage, INTEGRATIONS);
    if input.print == Print::Code || input.print == Print::All {
        eprintln!("{}", code);
    }
    code
}

fn find_used_templates(input: &TemplateInput, map: &mut HashMap<PathBuf, String>, source: String) {
    let mut check = vec![(input.path.clone(), source)];
    while let Some((path, source)) = check.pop() {
        for n in parse(&source, input.syntax) {
            match n {
                Node::Extends(Expr::StrLit(extends)) => {
                    let extends = input.config.find_template(extends, Some(&path));
                    let source = get_template_source(&extends);
                    check.push((extends, source));
                }
                Node::Import(_, import, _) => {
                    let import = input.config.find_template(import, Some(&path));
                    let source = get_template_source(&import);
                    check.push((import, source));
                }
                _ => {}
            }
        }
        map.insert(path, source);
    }
}

const INTEGRATIONS: Integrations = Integrations {
    actix: cfg!(feature = "actix-web"),
    gotham: cfg!(feature = "gotham"),
    iron: cfg!(feature = "iron"),
    rocket: cfg!(feature = "rocket"),
    warp: cfg!(feature = "warp"),
};
