#![feature(proc_macro, proc_macro_lib)]

#[macro_use]
extern crate nom;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use nom::IResult;
use proc_macro::TokenStream;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;

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

fn get_template_source(tpl_file: &str) -> String {
    let root = ::std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut path = PathBuf::from(root);
    path.push("templates");
    path.push(Path::new(tpl_file));
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn take_content(i: &[u8]) -> IResult<&[u8], &[u8]> {
    if i.len() < 1 || i[0] == b'{' {
        return IResult::Error(error_position!(nom::ErrorKind::TakeUntil, i));
    }
    for (j, c) in i.iter().enumerate() {
        if *c == b'{' {
            if i.len() < j + 2 {
                return IResult::Done(&i[..0], &i[..]);
            } else if i[j + 1] == '{' as u8 {
                return IResult::Done(&i[j..], &i[..j]);
            } else if i[j + 1] == '%' as u8 {
                return IResult::Done(&i[j..], &i[..j]);
            }
        }
    }
    IResult::Done(&i[..0], &i[..])
}

named!(var_expr, delimited!(tag!("{{"), take_until!("}}"), tag!("}}")));

named!(parse_template< Vec<&[u8]> >, many1!(alt!(take_content | var_expr)));

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    let ast = syn::parse_macro_input(&source).unwrap();
    let _ctx = match ast.body {
        syn::Body::Struct(ref data) => data,
        _ => panic!("#[derive(Template)] can only be used with structs"),
    };

    let name = &ast.ident;
    let path = get_path_from_attrs(&ast.attrs);
    let src = get_template_source(&path);

    let root = match parse_template(src.as_bytes()) {
        IResult::Done(_, res) => res,
        _ => panic!("problems parsing template source"),
    };

    let mut code = String::new();
    code.push_str("impl askama::Template for ");
    code.push_str(name.as_ref());
    code.push_str(" {\n");
    code.push_str("    fn render(&self) -> String {\n");
    code.push_str("        let mut buf = String::new();\n");
    code.push_str("        buf.push_str(\"");
    code.push_str(str::from_utf8(root[0]).unwrap());
    code.push_str("\");\n");
    code.push_str("        buf.push_str(&self.");
    code.push_str(str::from_utf8(root[1]).unwrap());
    code.push_str(");\n");
    code.push_str("        buf.push_str(\"");
    code.push_str(str::from_utf8(root[2]).unwrap());
    code.push_str("\");\n");
    code.push_str("        buf");
    code.push_str("    }\n");
    code.push_str("}\n\n");
    code.parse().unwrap()
}
