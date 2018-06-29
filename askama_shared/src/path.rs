use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use toml;

pub fn get_template_source(tpl_path: &Path) -> String {
    let mut path = template_dir();
    path.push(tpl_path);
    let mut f = match File::open(&path) {
        Err(_) => {
            let msg = format!("unable to open template file '{}'", &path.to_str().unwrap());
            panic!(msg)
        }
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
    let root = template_dir();
    if let Some(rel) = start_at {
        let mut fs_rel_path = root.clone();
        fs_rel_path.push(rel);
        fs_rel_path = fs_rel_path.with_file_name(path);
        if fs_rel_path.exists() {
            return fs_rel_path.strip_prefix(&root).unwrap().to_owned();
        }
    }

    let mut fs_abs_path = root.clone();
    let path = Path::new(path);
    fs_abs_path.push(Path::new(path));
    if fs_abs_path.exists() {
        path.to_owned()
    } else {
        panic!(format!(
            "template {:?} not found at {:?}",
            path.to_str().unwrap(),
            fs_abs_path
        ));
    }
}

pub fn template_dir() -> PathBuf {
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("templates");
    path
}

#[derive(Deserialize)]
struct Config {
    dirs: Option<Vec<String>>,
}

pub fn template_dirs() -> Vec<PathBuf> {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let askama = root.join("askama.toml");
    let default = vec![root.join("templates")];
    if askama.exists() {
        let mut contents = String::new();
        File::open(askama).unwrap().read_to_string(&mut contents).unwrap();
        let config: Config = toml::from_str(&contents).unwrap();
        if let Some(dirs) = config.dirs {
            let paths: Vec<PathBuf> = dirs.into_iter().map(|directory| {
                root.join(directory)
            }).collect();
            if paths.len() > 0 {
                paths
            } else {
                default
            }
        } else {
            default
        }
    } else {
        vec![root.join("templates")]
    }
}

#[cfg(test)]
mod tests {
    use super::Path;
    use super::{find_template_from_path, get_template_source};

    #[test]
    fn get_source() {
        assert_eq!(get_template_source(Path::new("sub/b.html")), "bar");
    }

    #[test]
    fn find_absolute() {
        let path = find_template_from_path("sub/b.html", Some(Path::new("a.html")));
        assert_eq!(path, Path::new("sub/b.html"));
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        find_template_from_path("b.html", Some(Path::new("a.html")));
    }

    #[test]
    fn find_relative() {
        let path = find_template_from_path("c.html", Some(Path::new("sub/b.html")));
        assert_eq!(path, Path::new("sub/c.html"));
    }

    #[test]
    fn find_relative_sub() {
        let path = find_template_from_path("sub1/d.html", Some(Path::new("sub/b.html")));
        assert_eq!(path, Path::new("sub/sub1/d.html"));
    }
}
