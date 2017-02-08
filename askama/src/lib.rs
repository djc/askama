#[macro_use]
extern crate nom;
extern crate syn;

pub trait Template {
    fn render_into(&self, writer: &mut std::fmt::Write);
    fn render(&self) -> String {
        let mut buf = String::new();
        self.render_into(&mut buf);
        buf
    }
}

mod generator;
mod parser;
mod path;

pub mod filters;
pub use path::rerun_if_templates_changed;
pub fn build_template(path: &str, ast: &syn::DeriveInput) -> String {
    let src = path::get_template_source(path);
    let nodes = parser::parse(&src);
    generator::generate(ast, path, nodes)
}
