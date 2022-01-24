#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use askama_shared::heritage::{Context, Heritage};
use askama_shared::input::{Print, Source, TemplateInput};
use askama_shared::parser::{parse, Expr, Node};
use askama_shared::{
    generator, get_template_source, read_config_file, CompileError, Config, Integrations,
};
use proc_macro::TokenStream;
use proc_macro2::Span;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match build_template(&ast) {
        Ok(source) => source.parse().unwrap(),
        Err(err) => syn::Error::from(err).to_compile_error().into(),
    }
}

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
fn build_template(ast: &syn::DeriveInput) -> Result<String, CompileError> {
    let config_toml = read_config_file().map_err(|msg| CompileError {
        msg,
        span: Span::call_site(),
    })?;
    let config = Config::new(&config_toml).map_err(|msg| CompileError {
        msg,
        span: Span::call_site(),
    })?;
    let input = TemplateInput::new(ast, &config)?;
    let (source, span) = match &input.source {
        Source::Source(s, span) => (s.clone(), span),
        Source::Path(_, span) => {
            let s = get_template_source(&input.path)
                .map_err(|msg| CompileError { msg, span: *span })?;
            (s, span)
        }
    };

    let mut sources = HashMap::new();
    find_used_templates(&input, &mut sources, source)
        .map_err(|msg| CompileError { msg, span: *span })?;

    let mut parsed = HashMap::new();
    for (path, src) in &sources {
        parsed.insert(
            path.as_path(),
            parse(src, input.syntax).map_err(|msg| CompileError { msg, span: *span })?,
        );
    }

    let mut contexts = HashMap::new();
    for (path, nodes) in &parsed {
        contexts.insert(
            *path,
            Context::new(input.config, path, nodes)
                .map_err(|msg| CompileError { msg, span: *span })?,
        );
    }

    let ctx = &contexts[input.path.as_path()];
    let heritage = if !ctx.blocks.is_empty() || ctx.extends.is_some() {
        Some(Heritage::new(ctx, &contexts))
    } else {
        None
    };

    if input.print == Print::Ast || input.print == Print::All {
        eprintln!("{:?}", parsed[input.path.as_path()]);
    }

    let code = generator::generate(&input, &contexts, heritage.as_ref(), INTEGRATIONS)
        .map_err(|msg| CompileError { msg, span: *span })?;
    if input.print == Print::Code || input.print == Print::All {
        eprintln!("{}", code);
    }
    Ok(code)
}

fn find_used_templates(
    input: &TemplateInput<'_>,
    map: &mut HashMap<PathBuf, String>,
    source: String,
) -> Result<(), Cow<'static, str>> {
    let mut dependency_graph = Vec::new();
    let mut check = vec![(input.path.clone(), source)];
    while let Some((path, source)) = check.pop() {
        for n in parse(&source, input.syntax)? {
            match n {
                Node::Extends(Expr::StrLit(extends)) => {
                    let extends = input.config.find_template(extends, Some(&path))?;
                    let dependency_path = (path.clone(), extends.clone());
                    if dependency_graph.contains(&dependency_path) {
                        return Err(format!(
                            "cyclic dependecy in graph {:#?}",
                            dependency_graph
                                .iter()
                                .map(|e| format!("{:#?} --> {:#?}", e.0, e.1))
                                .collect::<Vec<String>>(),
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

const INTEGRATIONS: Integrations = Integrations {
    actix: cfg!(feature = "actix-web"),
    axum: cfg!(feature = "axum"),
    gotham: cfg!(feature = "gotham"),
    mendes: cfg!(feature = "mendes"),
    rocket: cfg!(feature = "rocket"),
    tide: cfg!(feature = "tide"),
    warp: cfg!(feature = "warp"),
};
