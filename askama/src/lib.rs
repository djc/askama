//! Askama implements a type-safe compiler for Jinja-like templates.
//! It lets you write templates in a Jinja-like syntax,
//! which are linked to a `struct` defining the template context.
//! This is done using a custom derive implementation (implemented
//! in [`askama_derive`](https://crates.io/crates/askama_derive)).
//!
//! For feature highlights and a quick start, please review the
//! [README](https://github.com/djc/askama/blob/main/README.md).
//!
//! The primary documentation for this crate now lives in
//! [the book](https://djc.github.io/askama/).
//!
//! # Creating Askama templates
//!
//! An Askama template is a `struct` definition which provides the template
//! context combined with a UTF-8 encoded text file (or inline source, see
//! below). Askama can be used to generate any kind of text-based format.
//! The template file's extension may be used to provide content type hints.
//!
//! A template consists of **text contents**, which are passed through as-is,
//! **expressions**, which get replaced with content while being rendered, and
//! **tags**, which control the template's logic.
//! The template syntax is very similar to [Jinja](http://jinja.pocoo.org/),
//! as well as Jinja-derivatives like [Twig](http://twig.sensiolabs.org/) or
//! [Tera](https://github.com/Keats/tera).
//!
//! ## The `template()` attribute
//!
//! Askama works by generating one or more trait implementations for any
//! `struct` type decorated with the `#[derive(Template)]` attribute. The
//! code generation process takes some options that can be specified through
//! the `template()` attribute. The following sub-attributes are currently
//! recognized:
//!
//! * `path` (as `path = "foo.html"`): sets the path to the template file. The
//!   path is interpreted as relative to the configured template directories
//!   (by default, this is a `templates` directory next to your `Cargo.toml`).
//!   The file name extension is used to infer an escape mode (see below). In
//!   web framework integrations, the path's extension may also be used to
//!   infer the content type of the resulting response.
//!   Cannot be used together with `source`.
//! * `source` (as `source = "{{ foo }}"`): directly sets the template source.
//!   This can be useful for test cases or short templates. The generated path
//!   is undefined, which generally makes it impossible to refer to this
//!   template from other templates. If `source` is specified, `ext` must also
//!   be specified (see below). Cannot be used together with `path`.
//! * `ext` (as `ext = "txt"`): lets you specify the content type as a file
//!   extension. This is used to infer an escape mode (see below), and some
//!   web framework integrations use it to determine the content type.
//!   Cannot be used together with `path`.
//! * `print` (as `print = "code"`): enable debugging by printing nothing
//!   (`none`), the parsed syntax tree (`ast`), the generated code (`code`)
//!   or `all` for both. The requested data will be printed to stdout at
//!   compile time.
//! * `escape` (as `escape = "none"`): override the template's extension used for
//!   the purpose of determining the escaper for this template. See the section
//!   on configuring custom escapers for more information.
//! * `syntax` (as `syntax = "foo"`): set the syntax name for a parser defined
//!   in the configuration file. The default syntax , "default",  is the one
//!   provided by Askama.

#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama_derive::Template;
pub use askama_escape::{Html, Text};
pub use askama_shared::{
    self as shared, filters, helpers, DynTemplate, Error, MarkupDisplay, Result, Template,
};

/// Old build script helper to rebuild crates if contained templates have changed
///
/// This function is now deprecated and does nothing.
#[deprecated(
    since = "0.8.1",
    note = "file-level dependency tracking is handled automatically without build script"
)]
pub fn rerun_if_templates_changed() {}
