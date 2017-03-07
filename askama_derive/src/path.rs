use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

fn template_dir() -> PathBuf {
    let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("templates");
    path
}

pub fn get_template_source(tpl_file: &str) -> String {
    let mut path = template_dir();
    path.push(Path::new(tpl_file));
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
    s
}
