#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::fmt;
use std::{borrow::Cow, collections::HashMap};

use proc_macro::TokenStream;
use proc_macro2::Span;

use parser::ParseError;

mod config;
use config::Config;
mod generator;
use generator::{Generator, MapChain};
mod heritage;
use heritage::{Context, Heritage};
mod input;
use input::{Print, TemplateArgs, TemplateInput};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    match build_template(&ast) {
        Ok(source) => source.parse().unwrap(),
        Err(e) => e.into_compile_error(),
    }
}

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
pub(crate) fn build_template(ast: &syn::DeriveInput) -> Result<String, CompileError> {
    let template_args = TemplateArgs::new(ast)?;
    let toml = template_args.config()?;
    let config = Config::new(&toml, template_args.whitespace.as_deref())?;
    let input = TemplateInput::new(ast, &config, &template_args)?;

    let mut templates = HashMap::new();
    input.find_used_templates(&mut templates)?;

    let mut contexts = HashMap::new();
    for (path, parsed) in &templates {
        contexts.insert(
            path.as_path(),
            Context::new(input.config, path, parsed.nodes())?,
        );
    }

    let ctx = &contexts[input.path.as_path()];
    let heritage = if !ctx.blocks.is_empty() || ctx.extends.is_some() {
        Some(Heritage::new(ctx, &contexts))
    } else {
        None
    };

    if input.print == Print::Ast || input.print == Print::All {
        eprintln!("{:?}", templates[input.path.as_path()].nodes());
    }

    let code = Generator::new(&input, &contexts, heritage.as_ref(), MapChain::default())
        .build(&contexts[input.path.as_path()])?;
    if input.print == Print::Code || input.print == Print::All {
        eprintln!("{code}");
    }
    Ok(code)
}

#[derive(Debug, Clone)]
struct CompileError {
    msg: Cow<'static, str>,
    span: Span,
}

impl CompileError {
    fn new<S: Into<Cow<'static, str>>>(s: S, span: Span) -> Self {
        Self {
            msg: s.into(),
            span,
        }
    }

    fn into_compile_error(self) -> TokenStream {
        syn::Error::new(self.span, self.msg)
            .to_compile_error()
            .into()
    }
}

impl std::error::Error for CompileError {}

impl fmt::Display for CompileError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&self.msg)
    }
}

impl From<ParseError> for CompileError {
    #[inline]
    fn from(e: ParseError) -> Self {
        Self::new(e.to_string(), Span::call_site())
    }
}

impl From<&'static str> for CompileError {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::new(s, Span::call_site())
    }
}

impl From<String> for CompileError {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s, Span::call_site())
    }
}

// This is used by the code generator to decide whether a named filter is part of
// Askama or should refer to a local `filters` module. It should contain all the
// filters shipped with Askama, even the optional ones (since optional inclusion
// in the const vector based on features seems impossible right now).
const BUILT_IN_FILTERS: &[&str] = &[
    "abs",
    "capitalize",
    "center",
    "e",
    "escape",
    "filesizeformat",
    "fmt",
    "format",
    "indent",
    "into_f64",
    "into_isize",
    "join",
    "linebreaks",
    "linebreaksbr",
    "paragraphbreaks",
    "lower",
    "lowercase",
    "safe",
    "trim",
    "truncate",
    "upper",
    "uppercase",
    "urlencode",
    "urlencode_strict",
    "wordcount",
    // optional features, reserve the names anyway:
    "json",
    "markdown",
    "yaml",
];
