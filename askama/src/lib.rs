//! Askama implements a type-safe compiler for Jinja-like templates.
//! It lets you write templates in a Jinja-like syntax,
//! which are linked to a `struct` defining the template context.
//! This is done using a custom derive implementation (implemented
//! in [`askama_derive`](https://crates.io/crates/askama_derive)).
//!
//! For feature highlights and a quick start, please review the
//! [README](https://github.com/djc/askama/blob/master/README.md).
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
//! * `escape` (as `escape = "none"`): set the escape mode for expression
//!   output; the currently implemented modes are `none` and `html`. Askama
//!   infers the escape mode from the template file name (with `path`) or
//!   specified extension (`ext`): if the extension is `html`, `htm` or `xml`,
//!   the `html` escape mode is used; otherwise, no implicit escaping is done.
//!   Setting an escape mode explicitly overrides the inferred value.
//! * `syntax` (as `syntax = "foo"`): set the syntax name for a parser defined
//!   in the configuration file. The default syntax , "default",  is the one
//!   provided by Askama.
//!
//! ## Configuration
//!
//! At compile time, Askama will read optional configuration values from
//! `askama.toml` in the crate root (the directory where `Cargo.toml` can
//! be found). Currently, this covers the directories to search for templates,
//! as well as custom syntax configuration.
//!
//! This example file demonstrates the default configuration:
//!
//! ```toml
//! [general]
//! # Directories to search for templates, relative to the crate root.
//! dirs = ["templates"]
//! ```
//!
//! Here is an example that defines two custom syntaxes:
//!
//! ```toml
//! [general]
//! default_syntax = "foo"
//!
//! [[syntax]]
//! name = "foo"
//! block_start = "%{"
//! comment_start = "#{"
//! expr_end = "^^"
//!
//! [[syntax]]
//! name = "bar"
//! block_start = "%%"
//! block_end = "%%"
//! comment_start = "%#"
//! expr_start = "%{"
//! ```
//!
//! A syntax block consists of at least the attribute `name` which uniquely
//! names this syntax in the project.
//!
//! The following keys can currently be used to customize template syntax:
//!
//! * `block_start`, defaults to `{%`
//! * `block_end`, defaults to `%}`
//! * `comment_start`, defaults to `{#`
//! * `comment_end`, defaults to `#}`
//! * `expr_start`, defaults to `{{`
//! * `expr_end`, defaults to `}}`
//!
//! Values must be 2 characters long and start delimiters must all start with the same
//! character. If a key is omitted, the value from the default syntax is used.
//!
//! ## Variables
//!
//! Top-level template variables are defined by the template's context type.
//! You can use a dot (`.`) to access variable's attributes or methods.
//! Reading from variables is subject to the usual borrowing policies.
//! For example, `{{ name }}` will get the ``name`` field from the template
//! context,
//! while `{{ user.name }}` will get the ``name`` field of the ``user``
//! field from the template context.
//!
//! ## Assignments
//!
//! Inside code blocks, you can also declare variables or assign values
//! to variables.
//! Assignments can't be imported by other templates.
//!
//! Assignments use the let tag:
//!
//! ```text
//! {% let name = user.name %}
//! {% let len = name.len() %}
//!
//! {% let val -%}
//! {% if len == 0 -%}
//!   {% let val = "foo" -%}
//! {% else -%}
//!   {% let val = name -%}
//! {% endif -%}
//! {{ val }}
//! ```
//!
//! ## Filters
//!
//! Values such as those obtained from variables can be post-processed
//! using **filters**.
//! Filters are applied to values using the pipe symbol (`|`) and may
//! have optional extra arguments in parentheses.
//! Filters can be chained, in which case the output from one filter
//! is passed to the next.
//!
//! For example, `{{ "{:?}"|format(name|escape) }}` will escape HTML
//! characters from the value obtained by accessing the `name` field,
//! and print the resulting string as a Rust literal.
//!
//! The built-in filters are documented as part of the
//! [filters module documentation](filters/index.html).
//!
//! To define your own filters, simply have a module named `filters` in
//! scope of the context deriving a `Template` `impl`. Any filter names
//! that are not part of the built-in filters will be referenced through
//! the `filters::` prefix.
//!
//! ## Whitespace control
//!
//! Askama considers all tabs, spaces, newlines and carriage returns to be
//! whitespace. By default, it preserves all whitespace in template code,
//! except that a single trailing newline character is suppressed.
//! However, whitespace before and after expression and block delimiters
//! can be suppressed by writing a minus sign directly following a
//! start delimiter or leading into an end delimiter.
//!
//! Here is an example:
//!
//! ```text
//! {% if foo %}
//!   {{- bar -}}
//! {% else if -%}
//!   nothing
//! {%- endif %}
//! ```
//!
//! This discards all whitespace inside the if/else block. If a literal
//! (any part of the template not surrounded by `{% %}` or `{{ }}`)
//! includes only whitespace, whitespace suppression on either side will
//! completely suppress that literal content.
//!
//! ## Template inheritance
//!
//! Template inheritance allows you to build a base template with common
//! elements that can be shared by all inheriting templates.
//! A base template defines **blocks** that child templates can override.
//!
//! ### Base template
//!
//! ```text
//! <!DOCTYPE html>
//! <html lang="en">
//!   <head>
//!     <title>{% block title %}{{ title }} - My Site{% endblock %}</title>
//!     {% block head %}{% endblock %}
//!   </head>
//!   <body>
//!     <div id="content">
//!       {% block content %}{% endblock %}
//!     </div>
//!   </body>
//! </html>
//! ```
//!
//! The `block` tags define three blocks that can be filled in by child
//! templates. The base template defines a default version of the block.
//! A base template must define one or more blocks in order to enable
//! inheritance. Blocks can only be specified at the top level of a template
//! or inside other blocks, not inside `if`/`else` branches or in `for`-loop
//! bodies.
//!
//! ### Child template
//!
//! Here's an example child template:
//!
//! ```text
//! {% extends "base.html" %}
//!
//! {% block title %}Index{% endblock %}
//!
//! {% block head %}
//!   <style>
//!   </style>
//! {% endblock %}
//!
//! {% block content %}
//!   <h1>Index</h1>
//!   <p>Hello, world!</p>
//! {% endblock %}
//! ```
//!
//! The `extends` tag tells the code generator that this template inherits
//! from another template. It will search for the base template relative to
//! itself before looking relative to the template base directory. It will
//! render the top-level content from the base template, and substitute
//! blocks from the base template with those from the child template. Inside
//! a block in a child template, the `super()` macro can be called to render
//! the parent block's contents.
//!
//! ## HTML escaping
//!
//! Askama by default escapes variables if it thinks it is rendering HTML
//! content. It infers the escaping context from the extension of template
//! filenames, escaping by default if the extension is one of `html`, `htm`,
//! or `xml`. When specifying a template as `source` in an attribute, the
//! `ext` attribute parameter must be used to specify a type. Additionally,
//! you can specify an escape mode explicitly for your template by setting
//! the `escape` attribute parameter value (to `none` or `html`).
//!
//! Askama escapes `<`, `>`, `&`, `"`, `'`, `\` and `/`, according to the
//! [OWASP escaping recommendations][owasp]. Use the `safe` filter to
//! prevent escaping for a single expression, or the `escape` (or `e`)
//! filter to escape a single expression in an unescaped context.
//!
//! [owasp]: https://www.owasp.org/index.php/XSS_(Cross_Site_Scripting)_Prevention_Cheat_Sheet#RULE_.231_-_HTML_Escape_Before_Inserting_Untrusted_Data_into_HTML_Element_Content
//!
//! ## Control structures
//!
//! ### For
//!
//! Loop over each item in an iterator. For example:
//!
//! ```text
//! <h1>Users</h1>
//! <ul>
//! {% for user in users %}
//!   <li>{{ user.name|e }}</li>
//! {% endfor %}
//! </ul>
//! ```
//!
//! Inside for-loop blocks, some useful variables are accessible:
//!
//! * *loop.index*: current loop iteration (starting from 1)
//! * *loop.index0*: current loop iteration (starting from 0)
//! * *loop.first*: whether this is the first iteration of the loop
//!
//! ### If
//!
//! The *if* statement is used as you might expect:
//!
//! ```text
//! {% if users.len() == 0 %}
//!   No users
//! {% else if users.len() == 1 %}
//!   1 user
//! {% else %}
//!   {{ users.len() }} users
//! {% endif %}
//! ```
//!
//! ### Match
//!
//! In order to deal with Rust `enum`s in a type-safe way, templates support
//! match blocks from version 0.6. Here is a simple example showing how to
//! expand an `Option`:
//!
//! ```text
//! {% match item %}
//!   {% when Some with ("foo") %}
//!     Found literal foo
//!   {% when Some with (val) %}
//!     Found {{ val }}
//!   {% when None %}
//! {% endmatch %}
//! ```
//!
//! That is, a `match` block can optionally contain some whitespace (but
//! no other literal content), followed by a number of `when` blocks and
//! and an optional `else` block. Each `when` block must name a list of
//! matches (`(val)`), optionally introduced with a variant name. The
//! `else` block is equivalent to matching on `_` (matching anything).
//!
//! ### Include
//!
//! The *include* statement lets you split large or repetitive blocks into
//! separate template files. Included templates get full access to the context
//! in which they're used, including local variables like those from loops:
//!
//! ```text
//! {% for i in iter %}
//!   {% include "item.html" %}
//! {% endfor %}
//! ```
//!
//! ```text
//! * Item: {{ i }}
//! ```
//!
//! The path to include must be a string literal, so that it is known at
//! compile time. Askama will try to find the specified template relative
//! to the including template's path before falling back to the absolute
//! template path. Use `include` within the branches of an `if`/`else`
//! block to use includes more dynamically.
//!
//! ## Expressions
//!
//! Askama supports string literals (`"foo"`) and integer literals (`1`).
//! It supports almost all binary operators that Rust supports,
//! including arithmetic, comparison and logic operators.
//! The parser applies the same precedence order as the Rust compiler.
//! Expressions can be grouped using parentheses.
//! The HTML special characters `&`, `<` and `>` will be replaced with their
//! character entities unless the `escape` mode is disabled for a template.
//! Methods can be called on variables that are in scope, including `self`.
//!
//! **Warning**: if the result of an expression (a `{{ }}` block) is
//! equivalent to `self`, this can result in a stack overflow from infinite
//! recursion. This is because the `Display` implementation for that expression
//! will in turn evaluate the expression and yield `self` again.
//!
//! ## Comments
//!
//! Askama supports block comments delimited by `{#` and `#}`.
//!
//! # Optional functionality
//!
//! ## Rocket integration
//!
//! Enabling the `with-rocket` feature appends an implementation of Rocket's
//! `Responder` trait for each template type. This makes it easy to trivially
//! return a value of that type in a Rocket handler. See
//! [the example](https://github.com/djc/askama/blob/master/testing/tests/rocket.rs)
//! from the Askama test suite for more on how to integrate.
//!
//! In case a run-time error occurs during templating, a `500 Internal Server
//! Error` `Status` value will be returned, so that this can be further
//! handled by your error catcher.
//!
//! ## Iron integration
//!
//! Enabling the `with-iron` feature appends an implementation of Iron's
//! `Modifier<Response>` trait for each template type. This makes it easy to
//! trivially return a value of that type in an Iron handler. See
//! [the example](https://github.com/djc/askama/blob/master/testing/tests/iron.rs)
//! from the Askama test suite for more on how to integrate.
//!
//! Note that Askama's generated `Modifier<Response>` implementation currently
//! unwraps any run-time errors from the template. If you have a better
//! suggestion, please [file an issue](https://github.com/djc/askama/issues/new).
//!
//! ## Actix-web integration
//!
//! Enabling the `with-actix-web` feature appends an implementation of Actix-web's
//! `Responder` trait for each template type. This makes it easy to return a value of
//! that type in an Actix-web handler.
//!
//! ## Gotham integration
//!
//! Enabling the `with-gotham` feature appends an implementation of Gotham's `IntoResponse`
//! trait for each template type. This makes it easy to return a value of that type in a
//! Gotham handler.
//!
//! In case of a run-time error occurring during templating, the response will be of the same
//! signature, with a status code of `500 Internal Server Error`, mime `*/*`, and an empty `Body`.
//! This preserves the response chain if any custom error handling needs to occur.
//!
//! ## The `json` filter
//!
//! Enabling the `serde-json` filter will enable the use of the `json` filter.
//! This will output formatted JSON for any value that implements the required
//! `Serialize` trait.

#![allow(unused_imports)]
#[macro_use]
extern crate askama_derive;
use askama_shared as shared;

use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

/// Main `Template` trait; implementations are generally derived
pub trait Template {
    /// Helper method which allocates a new `String` and renders into it
    fn render(&self) -> Result<String> {
        let mut buf = String::new();
        self.render_into(&mut buf)?;
        Ok(buf)
    }
    /// Renders the template to the given `writer` buffer
    fn render_into(&self, writer: &mut dyn std::fmt::Write) -> Result<()>;
    /// Helper function to inspect the template's extension
    fn extension() -> Option<&'static str>
    where
        Self: Sized;
}

pub use crate::shared::filters;
pub use crate::shared::helpers;
pub use crate::shared::{read_config_file, Error, MarkupDisplay, Result};
pub use askama_derive::*;

#[cfg(feature = "with-iron")]
pub mod iron {
    extern crate iron;
    pub use self::iron::headers::ContentType;
    pub use self::iron::modifier::Modifier;
    pub use self::iron::response::Response;
}

#[cfg(feature = "with-rocket")]
pub mod rocket {
    extern crate rocket;

    use self::rocket::http::{ContentType, Status};
    pub use self::rocket::request::Request;
    use self::rocket::response::Response;
    use std::io::Cursor;

    pub use self::rocket::response::{Responder, Result};

    pub fn respond(t: &super::Template, ext: &str) -> Result<'static> {
        let rsp = t.render().map_err(|_| Status::InternalServerError)?;
        let ctype = ContentType::from_extension(ext).ok_or(Status::InternalServerError)?;
        Response::build()
            .header(ctype)
            .sized_body(Cursor::new(rsp))
            .ok()
    }
}

#[cfg(feature = "with-actix-web")]
pub mod actix_web {
    extern crate actix_web;
    extern crate mime_guess;

    // actix_web technically has this as a pub fn in later versions, fs::file_extension_to_mime.
    // Older versions that don't have it exposed are easier this way. If ext is empty or no
    // associated type was found, then this returns `application/octet-stream`, in line with how
    // actix_web handles it in newer releases.
    pub use self::actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
    use self::mime_guess::get_mime_type;

    pub fn respond(t: &super::Template, ext: &str) -> Result<HttpResponse, Error> {
        let rsp = t
            .render()
            .map_err(|_| ErrorInternalServerError("Template parsing error"))?;
        let ctype = get_mime_type(ext).to_string();
        Ok(HttpResponse::Ok().content_type(ctype.as_str()).body(rsp))
    }
}

#[cfg(feature = "with-gotham")]
pub mod gotham {
    pub use gotham::handler::IntoResponse;
    use gotham::helpers::http::response::{create_empty_response, create_response};
    pub use gotham::state::State;
    pub use hyper::{Body, Response, StatusCode};
    use mime_guess::get_mime_type;

    pub fn respond(t: &super::Template, ext: &str) -> Response<Body> {
        let mime_type = get_mime_type(ext).to_string();

        match t.render() {
            Ok(body) => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", mime_type.to_string())
                .body(body.into())
                .unwrap(),
            Err(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(vec![].into())
                .unwrap(),
        }
    }
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

/// Build script helper to rebuild crates if contained templates have changed
///
/// Iterates over all files in the template directories and writes a
/// `cargo:rerun-if-changed=` line for each of them to stdout.
///
/// This helper method can be used in build scripts (`build.rs`) in crates
/// that have templates, to make sure the crate gets rebuilt when template
/// source code changes.
pub fn rerun_if_templates_changed() {
    let file = read_config_file();
    for template_dir in &shared::Config::new(&file).dirs {
        visit_dirs(template_dir, &|e: &DirEntry| {
            println!("cargo:rerun-if-changed={}", e.path().to_str().unwrap());
        })
        .unwrap();
    }
}
