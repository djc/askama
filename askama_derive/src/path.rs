use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn get_template_source(tpl_path: &Path) -> String {
    let mut path = template_dir();
    path.push(tpl_path);
    let mut f = match File::open(&path) {
        Err(_) => {
            let msg = format!("unable to open template file '{}'",
                              &path.to_str().unwrap());
            panic!(msg);
        },
        Ok(f) => f,
    };
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    if s.ends_with('\n') {
        let _ = s.pop();
    }
    s
}

pub fn find_template_from_path<'a>(path: &str, start_at: Option<&str>) -> PathBuf {
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
        panic!(format!("template '{:?}' not found", path.to_str()));
    }
}

// Duplicated in askama
fn template_dir() -> PathBuf {
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("templates");
    path
}

#[cfg(test)]
mod tests {
    use super::{find_template_from_path, get_template_source};
    use super::Path;

    #[test]
    fn get_source() {
        assert_eq!(get_template_source(Path::new("sub/b.html")), "bar");
    }

    #[test]
    fn find_absolute() {
        let path = find_template_from_path("sub/b.html", Some("a.html"));
        assert_eq!(path, Path::new("sub/b.html"));
    }

    #[test]
    #[should_panic]
    fn find_relative_nonexistent() {
        find_template_from_path("b.html", Some("a.html"));
    }

    #[test]
    fn find_relative() {
        let path = find_template_from_path("c.html", Some("sub/b.html"));
        assert_eq!(path, Path::new("sub/c.html"));
    }

    #[test]
    fn find_relative_sub() {
        let path = find_template_from_path("sub1/d.html", Some("sub/b.html"));
        assert_eq!(path, Path::new("sub/sub1/d.html"));
    }
}
