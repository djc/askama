#![cfg_attr(feature = "cargo-clippy", allow(unused_parens))]

#[macro_use]
extern crate error_chain;

#[cfg(feature = "serde-json")]
extern crate serde;
#[cfg(feature = "serde-json")]
extern crate serde_json;

pub use escaping::MarkupDisplay;
pub use errors::{Error, Result};
pub mod filters;
pub mod path;

mod escaping;

mod errors {
    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
            Json(::serde_json::Error) #[cfg(feature = "serde-json")];
        }
    }
}
