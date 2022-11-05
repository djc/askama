#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::borrow::Cow;
use std::fmt;

use proc_macro2::{Span, TokenStream};

pub mod config;
pub mod generator;
pub mod input;
pub mod parser;

/// An error that occurred during compilation, along with the source location.
#[derive(Debug, Clone)]
pub struct CompileError {
    msg: Cow<'static, str>,
    span: Span,
}

impl CompileError {
    /// Create a new error, reporting a failure which is described by the message
    /// and occurred at the specified source location.
    pub fn new<S: Into<Cow<'static, str>>>(s: S, span: Span) -> Self {
        Self {
            msg: s.into(),
            span,
        }
    }

    /// Convert the error into a Rust compiler error.
    pub fn into_compile_error(self) -> TokenStream {
        syn::Error::new(self.span, self.msg).to_compile_error()
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
        Self::new(s, Span::call_site())
    }
}

impl From<String> for CompileError {
    #[inline]
    fn from(s: String) -> Self {
        Self::new(s, Span::call_site())
    }
}
