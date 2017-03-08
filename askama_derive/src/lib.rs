#[macro_use]
extern crate nom;
extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

mod generator;
mod parser;
mod path;

// Holds metadata for the template, based on the `template()` attribute.
struct TemplateMeta {
    path: String,
    print: String,
}

// Returns a `TemplateMeta` based on the `template()` attribute data found
// in the parsed struct or enum. Will panic if it does not find the required
// template path, or if the `print` key has an unexpected value.
fn get_template_meta(ast: &syn::DeriveInput) -> TemplateMeta {
    let attr = ast.attrs.iter().find(|a| a.name() == "template");
    if attr.is_none() {
        let msg = format!("'template' attribute not found on struct '{}'",
                          ast.ident.as_ref());
        panic!(msg);
    }

    let attr = attr.unwrap();
    let mut path = None;
    let mut print = "none".to_string();
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
fn build_template(ast: &syn::DeriveInput) -> String {
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

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };
    build_template(&ast).parse().unwrap()
}
