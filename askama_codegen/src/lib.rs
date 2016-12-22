#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    let ast = syn::parse_macro_input(&source).unwrap();
    let _ctx = match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics askama::Template for #name #ty_generics #where_clause {
             fn render(&self) -> String {
                 "hello world, bar".to_string()
             }
        }
    };

    expanded.parse().unwrap()
}
