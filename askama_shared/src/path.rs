use super::Config;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

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
    if let Some(root) = start_at {
        let relative = root.with_file_name(path);
        if relative.exists() {
            return relative.to_owned();
        }
    }

    let config = Config::new();
    for dir in &config.dirs {
        let rooted = dir.join(path);
        if rooted.exists() {
            return rooted;
        }
    }

    panic!("template {:?} not found in directories {:?}", path, config.dirs)
}

#[cfg(test)]
mod tests {
    use super::{find_template_from_path, get_template_source};

    #[test]
    fn get_source() {
        let path = find_template_from_path("sub/b.html", None);
        assert_eq!(get_template_source(&path), "bar");
    }

    #[test]
    fn find_absolute() {
        let root = find_template_from_path("a.html", None);
        let path = find_template_from_path("sub/b.html", Some(&root));
        assert!(path.to_str().unwrap().ends_with("sub/b.html"));
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        let root = find_template_from_path("a.html", None);
        find_template_from_path("b.html", Some(&root));
    }

    #[test]
    fn find_relative() {
        let root = find_template_from_path("sub/b.html", None);
        let path = find_template_from_path("c.html", Some(&root));
        assert!(path.to_str().unwrap().ends_with("sub/c.html"));
    }

    #[test]
    fn find_relative_sub() {
        let root = find_template_from_path("sub/b.html", None);
        let path = find_template_from_path("sub1/d.html", Some(&root));
        assert!(path.to_str().unwrap().ends_with("sub/sub1/d.html"));
    }
}
