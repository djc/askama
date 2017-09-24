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
    let imports = input::Imports::new(&nodes, &data.path);
    let imported_nodes = imports.parse();
    if data.meta.print == Print::Ast || data.meta.print == Print::All {
        println!("{:?}", nodes);
    }
    let code = generator::generate(&data, &nodes, &imported_nodes);
    if data.meta.print == Print::Code || data.meta.print == Print::All {
        println!("{}", code);
    }
    code
}

mod errors {
    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
            Json(::serde_json::Error) #[cfg(feature = "serde-json")];
        }
    }
}
