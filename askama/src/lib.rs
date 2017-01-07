#![feature(proc_macro)]

pub trait Template {
    fn render(&self) -> String;
}

pub mod filters;
