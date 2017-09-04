#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate nom;
extern crate quote;
extern crate syn;

#[cfg(feature = "serde-json")]
extern crate serde;
#[cfg(feature = "serde-json")]
extern crate serde_json;

pub use errors::{Error, Result};
pub mod filters;
pub mod path;
pub use parser::parse;
pub use generator::generate;

mod escaping;
mod generator;
mod parser;

mod errors {
    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
            Json(::serde_json::Error) #[cfg(feature = "serde-json")];
        }
    }
}
