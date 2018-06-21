extern crate askama_shared as shared;
#[macro_use]
extern crate nom;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod generator;
mod input;
mod parser;

use input::{Print, Source, TemplateInput};
use parser::{Expr, Macro, Node};
use proc_macro::TokenStream;
use shared::path;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match ast.data {
        syn::Data::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };
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
    let input = input::TemplateInput::new(ast);
    let source: String = match input.source {
        Source::Source(ref s) => s.clone(),
        Source::Path(_) => path::get_template_source(&input.path),
    };

    let mut sources = HashMap::new();
    find_used_templates(&mut sources, input.path.clone(), source);

    let mut parsed = HashMap::new();
    for (path, src) in &sources {
        parsed.insert(path, parser::parse(src));
    }

    let mut contexts = HashMap::new();
    for (path, nodes) in &parsed {
        contexts.insert(*path, Context::new(&input, nodes));
    }

    if input.print == Print::Ast || input.print == Print::All {
        println!("{:?}", parsed[&input.path]);
    }

    let code = generator::generate(&input, &contexts);
    if input.print == Print::Code || input.print == Print::All {
        println!("{}", code);
    }
    code
}

fn find_used_templates(map: &mut HashMap<PathBuf, String>, path: PathBuf, source: String) {
    let mut check = vec![(path, source)];
    while let Some((path, source)) = check.pop() {
        for n in parser::parse(&source) {
            match n {
                Node::Extends(Expr::StrLit(extends)) => {
                    let extends = path::find_template_from_path(extends, Some(&path));
                    let source = path::get_template_source(&extends);
                    check.push((extends, source));
                }
                Node::Import(_, import, _) => {
                    let import = path::find_template_from_path(import, Some(&path));
                    let source = path::get_template_source(&import);
                    check.push((import, source));
                }
                _ => {}
            }
        }
        map.insert(path, source);
    }
}

pub(crate) struct Context<'a> {
    nodes: &'a [Node<'a>],
    blocks: Vec<&'a Node<'a>>,
    macros: HashMap<&'a str, &'a Macro<'a>>,
    imports: HashMap<&'a str, PathBuf>,
    trait_name: String,
    derived: bool,
}

impl<'a> Context<'a> {
    fn new<'n>(input: &'n TemplateInput, nodes: &'n [Node<'n>]) -> Context<'n> {
        let mut base = None;
        let mut blocks = Vec::new();
        let mut macros = HashMap::new();
        let mut imports = HashMap::new();

        for n in nodes {
            match n {
                Node::Extends(Expr::StrLit(path)) => match base {
                    Some(_) => panic!("multiple extend blocks found"),
                    None => {
                        base = Some(*path);
                    }
                },
                def @ Node::BlockDef(_, _, _, _) => {
                    blocks.push(def);
                }
                Node::Macro(name, m) => {
                    macros.insert(*name, m);
                }
                Node::Import(_, import_path, scope) => {
                    let path = path::find_template_from_path(import_path, Some(&input.path));
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

        Context {
            nodes,
            blocks,
            macros,
            imports,
            trait_name: match base {
                Some(user_path) => trait_name_for_path(&path::find_template_from_path(
                    user_path,
                    Some(&input.path),
                )),
                None => trait_name_for_path(&input.path),
            },
            derived: base.is_some(),
        }
    }
}

fn trait_name_for_path(path: &Path) -> String {
    let mut res = String::new();
    res.push_str("TraitFrom");
    for c in path.to_string_lossy().chars() {
        if c.is_alphanumeric() {
            res.push(c);
        } else {
            res.push_str(&format!("{:x}", c as u32));
        }
    }
    res
}
