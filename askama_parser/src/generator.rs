use crate::config::get_template_source;
use crate::input::{Print, Source, TemplateInput};
use crate::parser::{parse, Node};
use crate::CompileError;

use std::collections::hash_map::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct TemplateArgs {
    pub source: Option<Source>,
    pub print: Print,
    pub escaping: Option<String>,
    pub ext: Option<String>,
    pub syntax: Option<String>,
    pub config_path: Option<String>,
}

impl TemplateArgs {
    pub fn new(ast: &'_ syn::DeriveInput) -> Result<Self, CompileError> {
        // Check that an attribute called `template()` exists once and that it is
        // the proper type (list).
        let mut template_args = None;
        for attr in &ast.attrs {
            let ident = match attr.path.get_ident() {
                Some(ident) => ident,
                None => continue,
            };

            if ident == "template" {
                if template_args.is_some() {
                    return Err("duplicated 'template' attribute".into());
                }

                match attr.parse_meta() {
                    Ok(syn::Meta::List(syn::MetaList { nested, .. })) => {
                        template_args = Some(nested);
                    }
                    Ok(_) => return Err("'template' attribute must be a list".into()),
                    Err(e) => return Err(format!("unable to parse attribute: {}", e).into()),
                }
            }
        }
        let template_args =
            template_args.ok_or_else(|| CompileError::from("no attribute 'template' found"))?;

        let mut args = Self::default();
        // Loop over the meta attributes and find everything that we
        // understand. Return a CompileError if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        for item in template_args {
            let pair = match item {
                syn::NestedMeta::Meta(syn::Meta::NameValue(ref pair)) => pair,
                _ => {
                    use quote::ToTokens;
                    return Err(format!(
                        "unsupported attribute argument {:?}",
                        item.to_token_stream()
                    )
                    .into());
                }
            };
            let ident = match pair.path.get_ident() {
                Some(ident) => ident,
                None => unreachable!("not possible in syn::Meta::NameValue(…)"),
            };

            if ident == "path" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    if args.source.is_some() {
                        return Err("must specify 'source' or 'path', not both".into());
                    }
                    args.source = Some(Source::Path(s.value()));
                } else {
                    return Err("template path must be string literal".into());
                }
            } else if ident == "source" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    if args.source.is_some() {
                        return Err("must specify 'source' or 'path', not both".into());
                    }
                    args.source = Some(Source::Source(s.value()));
                } else {
                    return Err("template source must be string literal".into());
                }
            } else if ident == "print" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    args.print = s.value().parse()?;
                } else {
                    return Err("print value must be string literal".into());
                }
            } else if ident == "escape" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    args.escaping = Some(s.value());
                } else {
                    return Err("escape value must be string literal".into());
                }
            } else if ident == "ext" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    args.ext = Some(s.value());
                } else {
                    return Err("ext value must be string literal".into());
                }
            } else if ident == "syntax" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    args.syntax = Some(s.value())
                } else {
                    return Err("syntax value must be string literal".into());
                }
            } else if ident == "config" {
                if let syn::Lit::Str(ref s) = pair.lit {
                    args.config_path = Some(s.value())
                } else {
                    return Err("config value must be string literal".into());
                }
            } else {
                return Err(format!("unsupported attribute key {:?} found", ident).into());
            }
        }

        Ok(args)
    }
}

pub fn find_used_templates(
    input: &TemplateInput<'_>,
    map: &mut HashMap<PathBuf, String>,
    source: String,
) -> Result<(), CompileError> {
    let mut dependency_graph = Vec::new();
    let mut check = vec![(input.path.clone(), source)];
    while let Some((path, source)) = check.pop() {
        for n in parse(&source, input.syntax)? {
            match n {
                Node::Extends(extends) => {
                    let extends = input.config.find_template(extends, Some(&path))?;
                    let dependency_path = (path.clone(), extends.clone());
                    if dependency_graph.contains(&dependency_path) {
                        return Err(format!(
                            "cyclic dependecy in graph {:#?}",
                            dependency_graph
                                .iter()
                                .map(|e| format!("{:#?} --> {:#?}", e.0, e.1))
                                .collect::<Vec<String>>()
                        )
                        .into());
                    }
                    dependency_graph.push(dependency_path);
                    let source = get_template_source(&extends)?;
                    check.push((extends, source));
                }
                Node::Import(_, import, _) => {
                    let import = input.config.find_template(import, Some(&path))?;
                    let source = get_template_source(&import)?;
                    check.push((import, source));
                }
                _ => {}
            }
        }
        map.insert(path, source);
    }
    Ok(())
}
