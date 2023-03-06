#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::borrow::Cow;
use std::fmt;

use proc_macro::TokenStream;

mod config;
mod generator;
mod heritage;
mod input;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    generator::derive_template(input)
}

#[derive(Debug, Clone)]
struct CompileError {
    msg: Cow<'static, str>,
}

impl CompileError {
    fn new<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Self { msg: s.into() }
    }
}

impl std::error::Error for CompileError {}

impl fmt::Display for CompileError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&self.msg)
    }
}

impl From<&'static str> for CompileError {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CompileError {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<askama_parser::ParseError> for CompileError {
    fn from(e: askama_parser::ParseError) -> Self {
        Self::new(e.to_string())
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
