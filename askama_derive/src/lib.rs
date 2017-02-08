extern crate askama;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;

fn get_path_from_attrs(attrs: &[syn::Attribute]) -> String {
    let attr = attrs.iter().find(|a| a.name() == "template").unwrap();
    if let syn::MetaItem::List(_, ref inner) = attr.value {
        if let syn::NestedMetaItem::MetaItem(ref item) = inner[0] {
            if let &syn::MetaItem::NameValue(ref key, ref val) = item {
                assert_eq!(key.as_ref(), "path");
                if let &syn::Lit::Str(ref s, _) = val {
                    return s.clone();
                }
            }
        }
    }
    panic!("template path not found in struct attributes");
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };
    let path = get_path_from_attrs(&ast.attrs);
    askama::build_template(&path, &ast).parse().unwrap()
}
