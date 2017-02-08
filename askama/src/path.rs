use std::env;
use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub fn get_template_source(tpl_file: &str) -> String {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut path = PathBuf::from(root);
    path.push("templates");
    path.push(Path::new(tpl_file));
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            let path = entry.path();
            if path.is_dir() {
                try!(visit_dirs(&path, cb));
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub fn rerun_if_templates_changed() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    visit_dirs(&Path::new(&root).join("templates"), &|e: &DirEntry| {
        println!("cargo:rerun-if-changed={}", e.path().to_str().unwrap());
    }).unwrap();
}
