#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::fmt;
use std::{borrow::Cow, collections::HashMap};

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use parser::ParseError;

mod config;
use config::Config;
mod generator;
use generator::{Generator, MapChain};
mod heritage;
use heritage::{Context, Heritage};
mod input;
use input::{Print, TemplateArgs, TemplateInput};
mod parse_proc_macro;
use parse_proc_macro::{get_generics_and_type_name, Generics, WhereClause};
pub(crate) mod unescape;

#[derive(Debug)]
pub(crate) struct DeriveInput {
    attrs: Vec<TokenStream>,
    ident: String,
    generics: Generics,
    where_clause: WhereClause,
}

impl DeriveInput {
    fn new(f: TokenStream) -> Self {
        let mut iter = f.into_iter();
        let mut attrs = Vec::new();
        let mut ident = None;
        let mut generics = Generics::default();
        let mut where_clause = WhereClause::default();

        // We're only interested into attributes. In the `TokenStream`, it's a `Punct` (`#`)
        // followed by a `Group` (with bracket delimiter);
        while let Some(next) = iter.next() {
            match next {
                TokenTree::Punct(p) if p.as_char() == '#' => {
                    match iter.next() {
                        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                            // This is an attribute, we store it.
                            attrs.push(g.stream());
                        }
                        _ => {}
                    }
                }
                TokenTree::Ident(id) => {
                    match id.to_string().as_str() {
                        "struct" | "enum" | "union" | "type" => {
                            get_generics_and_type_name(
                                &mut iter,
                                &mut ident,
                                &mut generics,
                                &mut where_clause,
                            );
                        }
                        // Very likely `pub`.
                        _ => {}
                    }
                }
                // FIXME: Normally, if it's anything else than a `#`, then we're done parsing.
                // Would be nice to check it's actually always the case.
                _ => {}
            }
        }
        Self {
            attrs,
            ident: ident.unwrap_or_default(),
            generics,
            where_clause,
        }
    }
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast = DeriveInput::new(input);
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
pub(crate) fn build_template(ast: &DeriveInput) -> Result<String, CompileError> {
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
    span: Option<Span>,
}

impl CompileError {
    fn new<S: Into<Cow<'static, str>>>(s: S, span: Option<Span>) -> Self {
        Self {
            msg: s.into(),
            span,
        }
    }

    fn into_compile_error(self) -> TokenStream {
        let span = self.span.expect("should not be run outside of proc-macro!");
        // We generate a `compile_error` macro and assign it to the current span so the error
        // displayed by rustc points to the right location.
        let mut stream = TokenStream::new();
        stream.extend(vec![TokenTree::Literal(Literal::string(&self.msg))]);

        let mut tokens = vec![
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("core", span)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("compile_error", span)),
            TokenTree::Punct(Punct::new('!', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, stream)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ];

        for tok in &mut tokens {
            tok.set_span(span);
        }

        TokenStream::from_iter(tokens)

        // let mut stream = TokenStream::new();
        // stream.extend(tokens);
        // let mut t = TokenTree::Group(Group::new(Delimiter::Brace, stream));
        // t.set_span(self.span);

        // let mut stream = TokenStream::new();
        // stream.extend(vec![t]);
        // stream
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
        if proc_macro::is_available() {
            Self::new(e.to_string(), Some(Span::call_site()))
        } else {
            Self::new(e.to_string(), None)
        }
    }
}

impl From<&'static str> for CompileError {
    #[inline]
    fn from(s: &'static str) -> Self {
        if proc_macro::is_available() {
            Self::new(s, Some(Span::call_site()))
        } else {
            Self::new(s, None)
        }
    }
}

impl From<String> for CompileError {
    #[inline]
    fn from(s: String) -> Self {
        if proc_macro::is_available() {
            Self::new(s, Some(Span::call_site()))
        } else {
            Self::new(s, None)
        }
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
