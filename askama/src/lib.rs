#[macro_use]
extern crate nom;

pub trait Template {
    fn render(&self) -> String;
}

pub mod filters;
pub mod generator;
pub mod parser;
