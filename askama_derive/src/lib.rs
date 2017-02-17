extern crate askama;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;

struct TemplateMeta {
    path: String,
}

fn get_path_from_attrs(attrs: &[syn::Attribute]) -> TemplateMeta {
    let mut path = None;
    let attr = attrs.iter().find(|a| a.name() == "template").unwrap();
    if let syn::MetaItem::List(_, ref inner) = attr.value {
        for nm_item in inner {
            if let &syn::NestedMetaItem::MetaItem(ref item) = nm_item {
                if let &syn::MetaItem::NameValue(ref key, ref val) = item {
                    match key.as_ref() {
                        "path" => if let &syn::Lit::Str(ref s, _) = val {
                            path = Some(s.clone());
                        } else {
                            panic!("template path must be string literal");
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
    TemplateMeta { path: path.unwrap() }
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };
    let meta = get_path_from_attrs(&ast.attrs);
    askama::build_template(&meta.path, &ast).parse().unwrap()
}
