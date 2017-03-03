//! Askama implements a type-safe compiler for Jinja-like templates.
//! It lets you write templates in a Jinja-like syntax,
//! which are linked to a `struct` defining the template context.
//! This is done using a custom derive implementation (implemented
//! in [askama_derive](https://crates.io/crates/askama_derive)).
//!
//! # Example template
//!
//! ```text
//! {% extends "layout.html" %}
//! {% block body %}
//!   <ul>
//!     {% for user in users %}
//!       <li><a href="{{ user.url }}">{{ user.username }}</a></li>
//!     {% endfor %}
//!   </ul>
//! {% endblock %}
//! ```
//!
//! # Feature highlights
//!
//! * Construct templates using a familiar, easy-to-use syntax
//! * Fully benefit from the safety provided by Rust's type system
//! * Templates do not perform eager conversion to strings or other types
//! * Template code is compiled into your crate for optimal performance
//! * Templates can directly access your Rust types, according to Rust's
//!   privacy rules
//! * Debugging features to assist you in template development
//! * Included filter functions will provide easy access to common functions
//! * Templates must be valid UTF-8 and produce UTF-8 when rendered
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
//! while `{{ user.name }}` will get the ``name`` field of the `user`
//! ``field`` of the template context.
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
//! of available filters. User-defined filters are currently not supported.
//!
//! ## Whitespace control
//!
//! Askama preserves all whitespace in template code by default,
//! except that final trailing newline characters are suppressed.
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
//! from another template. It will render the top-level content from the
//! base template, and substitute blocks from the base template with those
//! from the child template. The inheriting template context `struct` must
//! have a field called `_parent` of the type used as the base template
//! context. Blocks can only refer to the context of their own template.
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
//! ## Expressions
//!
//! Askama supports string literals (`"foo"`) and integer literals (`1`).
//! It supports almost all binary operators that Rust supports,
//! including arithmetic, comparison and logic operators.
//! The same precedence order as Rust uses is applied.
//! Expressions can be grouped using parentheses.

#[macro_use]
extern crate nom;
extern crate syn;

/// Main `Template` trait; implementations are generally derived
pub trait Template {
    /// Renders the template to the given `writer` buffer
    fn render_to(&self, writer: &mut std::fmt::Write);
    /// Helper method which allocates a new `String` and renders into it
    fn render(&self) -> String {
        let mut buf = String::new();
        self.render_to(&mut buf);
        buf
    }
}

mod generator;
mod parser;
mod path;

pub mod filters;
pub use path::rerun_if_templates_changed;

// Holds metadata for the template, based on the `template()` attribute.
struct TemplateMeta {
    path: String,
    print: String,
}

// Returns a `TemplateMeta` based on the `template()` attribute data found
// in the parsed struct or enum. Will panic if it does not find the required
// template path, or if the `print` key has an unexpected value.
fn get_template_meta(ast: &syn::DeriveInput) -> TemplateMeta {
    let mut path = None;
    let mut print = "none".to_string();
    let attr = ast.attrs.iter().find(|a| a.name() == "template").unwrap();
    if let syn::MetaItem::List(_, ref inner) = attr.value {
        for nm_item in inner {
            if let syn::NestedMetaItem::MetaItem(ref item) = *nm_item {
                if let syn::MetaItem::NameValue(ref key, ref val) = *item {
                    match key.as_ref() {
                        "path" => if let syn::Lit::Str(ref s, _) = *val {
                            path = Some(s.clone());
                        } else {
                            panic!("template path must be string literal");
                        },
                        "print" => if let syn::Lit::Str(ref s, _) = *val {
                            print = s.clone();
                        } else {
                            panic!("print value must be string literal");
                        },
                        _ => { panic!("unsupported annotation key found") }
                    }
                }
            }
        }
    }
    if path.is_none() {
        panic!("template path not found in struct attributes");
    }
    TemplateMeta { path: path.unwrap(), print: print }
}

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
pub fn build_template(ast: &syn::DeriveInput) -> String {
    let meta = get_template_meta(ast);
    let mut src = path::get_template_source(&meta.path);
    if src.ends_with('\n') {
        let _ = src.pop();
    }
    let nodes = parser::parse(&src);
    if meta.print == "ast" || meta.print == "all" {
        println!("{:?}", nodes);
    }
    let code = generator::generate(ast, &meta.path, nodes);
    if meta.print == "code" || meta.print == "all" {
        println!("{}", code);
    }
    code
}
