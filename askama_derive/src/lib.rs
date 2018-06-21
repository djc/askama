extern crate askama_shared as shared;
#[macro_use]
extern crate nom;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

mod generator;
mod input;
mod parser;

use input::{Print, Source};
use proc_macro::TokenStream;
use shared::path;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match ast.data {
        syn::Data::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };
    build_template(&ast).parse().unwrap()
}

/// Takes a `syn::DeriveInput` and generates source code for it
///
/// Reads the metadata from the `template()` attribute to get the template
/// metadata, then fetches the source from the filesystem. The source is
/// parsed, and the parse tree is fed to the code generator. Will print
/// the parse tree and/or generated source according to the `print` key's
/// value as passed to the `template()` attribute.
fn build_template(ast: &syn::DeriveInput) -> String {
    let input = input::TemplateInput::new(ast);
    let source = match input.source {
        Source::Source(ref s) => s.clone(),
        Source::Path(_) => path::get_template_source(&input.path),
    };

    let nodes = parser::parse(&source);
    if input.print == Print::Ast || input.print == Print::All {
        println!("{:?}", nodes);
    }

    let code = generator::generate(&input, &nodes);
    if input.print == Print::Code || input.print == Print::All {
        println!("{}", code);
    }
    code
}
