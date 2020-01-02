extern crate proc_macro;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

mod generator;
mod input;
mod parser;

use crate::input::{Print, Source, TemplateInput};
use crate::parser::{Expr, Macro, Node};
use askama_shared::{read_config_file, Config};
use proc_macro::TokenStream;

use crate::parser::parse;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    build_template(&ast).parse().unwrap()
}

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
fn build_template(ast: &syn::DeriveInput) -> String {
    let config_toml = read_config_file();
    let config = Config::new(&config_toml);
    let input = TemplateInput::new(ast, &config);
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

    let code = generator::generate(&input, &contexts, &heritage);
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

pub(crate) struct Heritage<'a> {
    root: &'a Context<'a>,
    blocks: BlockAncestry<'a>,
}

impl<'a> Heritage<'a> {
    fn new<'n>(
        mut ctx: &'n Context<'n>,
        contexts: &'n HashMap<&'n PathBuf, Context<'n>>,
    ) -> Heritage<'n> {
        let mut blocks: BlockAncestry<'n> = ctx
            .blocks
            .iter()
            .map(|(name, def)| (*name, vec![(ctx, *def)]))
            .collect();

        while let Some(ref path) = ctx.extends {
            ctx = &contexts[&path];
            for (name, def) in &ctx.blocks {
                blocks
                    .entry(name)
                    .or_insert_with(|| vec![])
                    .push((ctx, def));
            }
        }

        Heritage { root: ctx, blocks }
    }
}

type BlockAncestry<'a> = HashMap<&'a str, Vec<(&'a Context<'a>, &'a Node<'a>)>>;

pub(crate) struct Context<'a> {
    nodes: &'a [Node<'a>],
    extends: Option<PathBuf>,
    blocks: HashMap<&'a str, &'a Node<'a>>,
    macros: HashMap<&'a str, &'a Macro<'a>>,
    imports: HashMap<&'a str, PathBuf>,
}

impl<'a> Context<'a> {
    fn new<'n>(config: &Config, path: &Path, nodes: &'n [Node<'n>]) -> Context<'n> {
        let mut extends = None;
        let mut blocks = Vec::new();
        let mut macros = HashMap::new();
        let mut imports = HashMap::new();

        for n in nodes {
            match n {
                Node::Extends(Expr::StrLit(extends_path)) => match extends {
                    Some(_) => panic!("multiple extend blocks found"),
                    None => {
                        extends = Some(config.find_template(extends_path, Some(path)));
                    }
                },
                def @ Node::BlockDef(_, _, _, _) => {
                    blocks.push(def);
                }
                Node::Macro(name, m) => {
                    macros.insert(*name, m);
                }
                Node::Import(_, import_path, scope) => {
                    let path = config.find_template(import_path, Some(path));
                    imports.insert(*scope, path);
                }
                _ => {}
            }
        }

        let mut check_nested = 0;
        let mut nested_blocks = Vec::new();
        while check_nested < blocks.len() {
            if let Node::BlockDef(_, _, ref nodes, _) = blocks[check_nested] {
                for n in nodes {
                    if let def @ Node::BlockDef(_, _, _, _) = n {
                        nested_blocks.push(def);
                    }
                }
            } else {
                panic!("non block found in list of blocks");
            }
            blocks.append(&mut nested_blocks);
            check_nested += 1;
        }

        let blocks: HashMap<_, _> = blocks
            .iter()
            .map(|def| {
                if let Node::BlockDef(_, name, _, _) = def {
                    (*name, *def)
                } else {
                    unreachable!()
                }
            })
            .collect();

        Context {
            nodes,
            extends,
            blocks,
            macros,
            imports,
        }
    }
}

#[allow(clippy::match_wild_err_arm)]
fn get_template_source(tpl_path: &Path) -> String {
    match fs::read_to_string(tpl_path) {
        Err(_) => panic!(
            "unable to open template file '{}'",
            tpl_path.to_str().unwrap()
        ),
        Ok(mut source) => {
            if source.ends_with('\n') {
                let _ = source.pop();
            }
            source
        }
    }
}

#[cfg(test)]
mod tests {
    use super::get_template_source;
    use crate::Config;

    #[test]
    fn get_source() {
        let path = Config::new("").find_template("b.html", None);
        assert_eq!(get_template_source(&path), "bar");
    }
}
