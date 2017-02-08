extern crate askama;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;

fn get_path_from_attrs(attrs: &Vec<syn::Attribute>) -> String {
    for attr in attrs {
        if attr.name() == "template" {
            match attr.value {
                syn::MetaItem::List(_, ref inner) => {
                    match inner[0] {
                        syn::NestedMetaItem::MetaItem(ref item) => {
                            match item {
                                &syn::MetaItem::NameValue(ref key, ref val) => {
                                    assert_eq!(key.as_ref(), "path");
                                    match val {
                                        &syn::Lit::Str(ref s, _) => { return s.clone(); },
                                        _ => panic!("template path must be a string"),
                                    }
                                },
                                _ => panic!("template annotation must contain key/value pair"),
                            }
                        },
                        _ => panic!("template annotation must contain item"),
                    }
                },
                _ => panic!("template annotation must be of List type"),
            }
        }
    }
    panic!("template annotation not found");
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    let ast = syn::parse_derive_input(&source).unwrap();
    let _ctx = match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };

    let path = get_path_from_attrs(&ast.attrs);
    askama::build_template(&path, &ast).parse().unwrap()
}
