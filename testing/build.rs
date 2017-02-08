use std::env;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

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

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    visit_dirs(&Path::new(&root).join("templates"), &|e: &DirEntry| {
        println!("cargo:rerun-if-changed={}", e.path().to_str().unwrap());
    }).unwrap();
}
