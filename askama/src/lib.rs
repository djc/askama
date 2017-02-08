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

pub mod filters;
pub mod generator;
pub mod parser;
mod path;
pub use path::rerun_if_templates_changed;
