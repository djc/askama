use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use toml;

fn search_template_in_dirs<'a>(dirs: &'a [PathBuf], path: &Path) -> Option<&'a PathBuf> {
    dirs.iter().find(|dir| dir.join(path).exists())
}

pub fn get_template_source(tpl_path: &Path) -> String {
    let mut f = match File::open(tpl_path) {
        Err(_) => panic!("unable to open template file '{}'", tpl_path.to_str().unwrap()),
        Ok(f) => f,
    };

    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    if s.ends_with('\n') {
        let _ = s.pop();
    }
    s
}

pub fn find_template_from_path(path: &str, start_at: Option<&Path>) -> PathBuf {
    let dirs = template_dirs();
    if let Some(rel) = start_at {
        let root = search_template_in_dirs(&dirs, rel).expect(&format!(
            "unable to find previously available template file '{}'",
            rel.to_str().unwrap()
        ));
        let fs_rel_path = root.join(rel).with_file_name(path);
        if fs_rel_path.exists() {
            return fs_rel_path.to_owned();
        }
    }

    let mut rooted = PathBuf::new();
    match search_template_in_dirs(&dirs, &Path::new(path)) {
        Some(dir) => rooted.push(&dir),
        None => panic!("template {:?} not found in directories {:?}", path, dirs),
    };
    rooted.push(path);
    rooted
}

pub fn template_dirs() -> Vec<PathBuf> {
    Config::from_file().dirs
}

struct Config {
    dirs: Vec<PathBuf>,
}

impl Config {
    fn from_file() -> Config {
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

#[cfg(test)]
mod tests {
    use super::Path;
    use super::{find_template_from_path, get_template_source};

    #[test]
    fn get_source() {
        let path = find_template_from_path("sub/b.html", None);
        assert_eq!(get_template_source(&path), "bar");
    }

    #[test]
    fn find_absolute() {
        let path = find_template_from_path("sub/b.html", Some(Path::new("a.html")));
        assert!(path.to_str().unwrap().ends_with("sub/b.html"));
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        find_template_from_path("b.html", Some(Path::new("a.html")));
    }

    #[test]
    fn find_relative() {
        let path = find_template_from_path("c.html", Some(Path::new("sub/b.html")));
        assert!(path.to_str().unwrap().ends_with("sub/c.html"));
    }

    #[test]
    fn find_relative_sub() {
        let path = find_template_from_path("sub1/d.html", Some(Path::new("sub/b.html")));
        assert!(path.to_str().unwrap().ends_with("sub/sub1/d.html"));
    }
}
