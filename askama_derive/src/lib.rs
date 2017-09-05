extern crate askama_shared as shared;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };
    shared::build_template(&ast).parse().unwrap()
}
