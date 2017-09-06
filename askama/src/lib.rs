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
//! An Askama template is just a text file, in the UTF-8 encoding.
//! It can be used to generate any kind of text-based format.
//! You can use whatever extension you like.
//!
//! A template consists of **text contents**, which are passed through as-is,
//! **expressions**, which get replaced with content while being rendered, and
//! **tags**, which control the template's logic.
//! The template syntax is very similar to [Jinja](http://jinja.pocoo.org/),
//! as well as Jinja-derivatives like [Twig](http://twig.sensiolabs.org/) or
//! [Tera](https://github.com/Keats/tera).
//!
//! ## Variables
//!
//! Template variables are defined by the template context linked to the
//! template by deriving a `struct`'s implementation for `Template` from
//! a template file's contents.
//! You can use a dot (`.`) to get a variable's attributes.
//! Reading from variables is subject to the usual borrowing policies.
//! For example, `{{ name }}` will get the ``name`` field from the template
//! context,
//! while `{{ user.name }}` will get the ``name`` field of the ``user``
//! field from the template context.
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
//! Consult the [filters module documentation](filters/index.html) for a list
//! of available filters.
//!
//! ## Whitespace control
//!
//! Askama preserves all whitespace in template code by default,
//! except that a single trailing newline characters are suppressed.
//! However, whitespace before and after expression and block delimiters
//! can be suppressed by writing a minus sign directly following a
//! start delimiter or leading into an end delimiter.
//! Askama considers all tabs, spaces, newlines and carriage returns to be
//! whitespace.
//!
//! ## Template inheritance
//!
//! Template inheritance allows you to build a base template with common
//! elements that can then be shared by all inheriting templates.
//! A base template defines **blocks** that child templates can then override.
//!
//! ### Base template
//!
//! ```text
//! <!DOCTYPE html>
//! <html lang="en">
//!   <head>
//!     <title>{{ block title %}{{ title }}{% endblock %} - My Site</title>
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
//! A base template must define one or more blocks in order to be enable
//! inheritance. Blocks can only be specified at the top level of a template,
//! not inside `if`/`else` branches or in `for`-loop bodies.
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
//! blocks from the base template with those from the child template. The
//! inheriting template context `struct` must have a field called `_parent` of
//! the type used as the base template context. Blocks can refer to the context
//! of both parent and child template.
//!
//! Note that, if the base template lives in another module than the child
//! template, the child template's module should import all symbols from the
//! base template's module in order for it to find the trait definition that
//! supports the inheritance mechanism.
//!
//! ## HTML escaping
//!
//! Askama does not yet support automatic escaping. Care must be taken to
//! escape content that may contain HTML control characters. You can use
//! the `escape` filter (or its `e` alias) to escape data for use in HTML.
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

#![allow(unused_imports)]
#[macro_use]
extern crate askama_derive;
extern crate askama_shared as shared;

use shared::path;

use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

/// Main `Template` trait; implementations are generally derived
pub trait Template {
    /// Renders the template to the given `writer` buffer
    fn render_into(&self, writer: &mut std::fmt::Write) -> Result<()>;
    /// Helper method which allocates a new `String` and renders into it
    fn render(&self) -> Result<String> {
        let mut buf = String::new();
        self.render_into(&mut buf)?;
        Ok(buf)
    }
}

pub use shared::filters;
pub use askama_derive::*;
pub use shared::{Error, MarkupDisplay, Result};

#[cfg(feature = "with-iron")]
pub mod iron {
    extern crate iron;
    pub use self::iron::modifier::Modifier;
    pub use self::iron::response::Response;
}

#[cfg(feature = "with-rocket")]
pub mod rocket {
    extern crate rocket;
    pub use self::rocket::http::{ContentType, Status};
    pub use self::rocket::request::Request;
    pub use self::rocket::response::{Responder, Response};
}

fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            let path = entry.path();
            if path.is_dir() {
                try!(visit_dirs(&path, cb));
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

/// Build script helper to rebuild crates if contained templates have changed
///
/// Iterates over all files in the template dir (`templates` in
/// `CARGO_MANIFEST_DIR`) and writes a `cargo:rerun-if-changed=` line for each
/// of them to stdout.
///
/// This helper method can be used in build scripts (`build.rs`) in crates
/// that have templates, to make sure the crate gets rebuilt when template
/// source code changes.
pub fn rerun_if_templates_changed() {
    visit_dirs(&path::template_dir(), &|e: &DirEntry| {
        println!("cargo:rerun-if-changed={}", e.path().to_str().unwrap());
    }).unwrap();
}
