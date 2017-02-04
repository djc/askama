#[macro_use]
extern crate nom;
extern crate syn;

pub trait Template {
    fn render(&self) -> String;
}

pub mod filters;
pub mod generator;
pub mod parser;
