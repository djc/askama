use std::fs;
use std::path::Path;

pub fn get_template_source(tpl_path: &Path) -> String {
    match fs::read_to_string(tpl_path) {
        Err(_) => panic!(
            "unable to open template file '{}'",
            tpl_path.to_str().unwrap()
        ),
        Ok(mut source) => {
            if source.ends_with('\n') {
                let _ = source.pop();
            }
            source
        }
    }
}

#[cfg(test)]
mod tests {
    use super::get_template_source;
    use Config;

    #[test]
    fn get_source() {
        let path = Config::new().find_template("sub/b.html", None);
        assert_eq!(get_template_source(&path), "bar");
    }
}
