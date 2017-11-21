#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate nom;
extern crate quote;
extern crate syn;

#[cfg(feature = "serde-json")]
extern crate serde;
#[cfg(feature = "serde-json")]
extern crate serde_json;

pub use escaping::MarkupDisplay;
pub use errors::{Error, Result};
pub mod filters;
pub mod path;

mod escaping;
mod generator;
mod input;
mod parser;

use input::Print;
use parser::{Macro, Node};

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
pub fn build_template(ast: &syn::DeriveInput) -> String {
    let data = input::TemplateInput::new(ast);
    let nodes = parser::parse(data.source.as_ref());
    let imports = Imports::new(&nodes, &data.path);
    if data.meta.print == Print::Ast || data.meta.print == Print::All {
        println!("{:?}", nodes);
    }
    let code = generator::generate(&data, &nodes, &imports.macro_map());
    if data.meta.print == Print::Code || data.meta.print == Print::All {
        println!("{}", code);
    }
    code
}


pub struct Imports<'a> {
    pub sources: HashMap<&'a str, Cow<'a, str>>,
}

impl<'a> Imports<'a> {
    pub fn new(parent_nodes: &'a [Node], parent_path: &'a Path) -> Imports<'a> {
        let sources = parent_nodes.iter().filter_map(|n| {
            match *n {
                Node::Import(_, ref import_path, scope) => {
                    let path = path::find_template_from_path(import_path, Some(parent_path));
                    let src = path::get_template_source(&path);
                    Some((scope, Cow::Owned(src)))
                },
                _ => None,
            }
        }).collect();
        Imports { sources }
    }

    pub fn macro_map(&'a self) -> HashMap<(&'a str, &'a str), Macro<'a>> {
        let mut macro_map = HashMap::new();
        for (scope, s) in self.sources.iter() {
            for n in parser::parse(&s.as_ref()) {
                match n {
                    Node::Macro(name, m) => macro_map.insert((*scope, name), m),
                    _ => None,
                };
            }
        }
        macro_map
    }
}


mod errors {
    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
            Json(::serde_json::Error) #[cfg(feature = "serde-json")];
        }
    }
}
