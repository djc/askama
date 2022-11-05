#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use proc_macro::TokenStream;

use askama_parser::{config, input, parser, CompileError};

mod generator;
mod heritage;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    generator::derive_template(input)
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
