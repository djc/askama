#![cfg_attr(feature = "cargo-clippy", allow(unused_parens))]

#[cfg(feature = "serde-json")]
extern crate serde;
#[cfg(feature = "serde-json")]
extern crate serde_json;

pub use escaping::MarkupDisplay;
pub use error::{Error, Result};
mod error;
pub mod filters;
pub mod path;

mod escaping;
