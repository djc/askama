#![cfg_attr(feature = "cargo-clippy", allow(unused_parens))]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde-json")]
extern crate serde_json;
extern crate toml;

use std::env;
use std::fs;
use std::path::PathBuf;

mod escaping;
mod error;

pub use error::{Error, Result};
pub use escaping::MarkupDisplay;
pub mod filters;
pub mod path;

struct Config {
    dirs: Vec<PathBuf>,
}

impl Config {
    fn new() -> Config {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let filename = root.join(CONFIG_FILE_NAME);

        let default = vec![root.join("templates")];
        let dirs = if filename.exists() {
            let config_str = fs::read_to_string(&filename)
                .expect(&format!("unable to read {}", filename.to_str().unwrap()));
            let raw: RawConfig = toml::from_str(&config_str)
                .expect(&format!("invalid TOML in {}", filename.to_str().unwrap()));
            raw.dirs
                .map(|dirs| dirs.into_iter().map(|dir| root.join(dir)).collect())
                .unwrap_or_else(|| default)
        } else {
            default
        };

        Config { dirs }
    }
}

#[derive(Deserialize)]
struct RawConfig {
    dirs: Option<Vec<String>>,
}

static CONFIG_FILE_NAME: &str = "askama.toml";
