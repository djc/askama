#![cfg_attr(feature = "cargo-clippy", allow(unused_parens))]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde-json")]
extern crate serde_json;
extern crate toml;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod error;
mod escaping;

pub use error::{Error, Result};
pub use escaping::MarkupDisplay;
pub mod filters;
pub mod path;

pub struct Config {
    pub dirs: Vec<PathBuf>,
}

impl Config {
    pub fn new() -> Config {
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

    pub fn find_template(&self, path: &str, start_at: Option<&Path>) -> PathBuf {
        if let Some(root) = start_at {
            let relative = root.with_file_name(path);
            if relative.exists() {
                return relative.to_owned();
            }
        }

        for dir in &self.dirs {
            let rooted = dir.join(path);
            if rooted.exists() {
                return rooted;
            }
        }

        panic!(
            "template {:?} not found in directories {:?}",
            path, self.dirs
        )
    }
}

#[derive(Deserialize)]
struct RawConfig {
    dirs: Option<Vec<String>>,
}

static CONFIG_FILE_NAME: &str = "askama.toml";

#[cfg(test)]
mod tests {
    use super::Config;
    use std::env;
    use std::path::{Path, PathBuf};

    fn assert_eq_rooted(actual: &Path, expected: &str) {
        let mut root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        root.push("templates");
        let mut inner = PathBuf::new();
        inner.push(expected);
        assert_eq!(actual.strip_prefix(root).unwrap(), inner);
    }

    #[test]
    fn find_absolute() {
        let config = Config::new();
        let root = config.find_template("a.html", None);
        let path = config.find_template("sub/b.html", Some(&root));
        assert_eq_rooted(&path, "sub/b.html");
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        let config = Config::new();
        let root = config.find_template("a.html", None);
        config.find_template("b.html", Some(&root));
    }

    #[test]
    fn find_relative() {
        let config = Config::new();
        let root = config.find_template("sub/b.html", None);
        let path = config.find_template("c.html", Some(&root));
        assert_eq_rooted(&path, "sub/c.html");
    }

    #[test]
    fn find_relative_sub() {
        let config = Config::new();
        let root = config.find_template("sub/b.html", None);
        let path = config.find_template("sub1/d.html", Some(&root));
        assert_eq_rooted(&path, "sub/sub1/d.html");
    }
}
