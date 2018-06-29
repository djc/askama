#![cfg_attr(feature = "cargo-clippy", allow(unused_parens))]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde-json")]
extern crate serde_json;
extern crate toml;

pub use error::{Error, Result};
pub use escaping::MarkupDisplay;
mod error;
pub mod filters;
pub mod path;

mod escaping;
