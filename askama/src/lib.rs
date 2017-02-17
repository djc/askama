#[macro_use]
extern crate nom;
extern crate syn;

pub trait Template {
    fn render_to(&self, writer: &mut std::fmt::Write);
    fn render(&self) -> String {
        let mut buf = String::new();
        self.render_to(&mut buf);
        buf
    }
}

mod generator;
mod parser;
mod path;

pub mod filters;
pub use path::rerun_if_templates_changed;

struct TemplateMeta {
    path: String,
}

fn get_template_meta(ast: &syn::DeriveInput) -> TemplateMeta {
    let mut path = None;
    let attr = ast.attrs.iter().find(|a| a.name() == "template").unwrap();
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

pub fn build_template(ast: &syn::DeriveInput) -> String {
    let meta = get_template_meta(ast);
    let src = path::get_template_source(&meta.path);
    let nodes = parser::parse(&src);
    generator::generate(ast, &meta.path, nodes)
}
