//! Module for built-in filter functions
//!
//! Contains all the built-in filter functions for use in templates.
//! Currently, there is no way to define filters outside this module.
use std::fmt;

fn escapable(b: &u8) -> bool {
    *b == b'<' || *b == b'>' || *b == b'&'
}

/// Escapes `&`, `<` and `>` in strings
pub fn escape(s: &fmt::Display) -> String {
    let s = format!("{}", s);
    let mut found = Vec::new();
    for (i, b) in s.as_bytes().iter().enumerate() {
        if escapable(b) {
            found.push(i);
        }
    }
    if found.is_empty() {
        return s;
    }

    let bytes = s.as_bytes();
    let max_len = bytes.len() + found.len() * 3;
    let mut res = Vec::<u8>::with_capacity(max_len);
    let mut start = 0;
    for idx in &found {
        if start < *idx {
            res.extend(&bytes[start..*idx]);
        }
        start = *idx + 1;
        match bytes[*idx] {
            b'<' => { res.extend(b"&lt;"); },
            b'>' => { res.extend(b"&gt;"); },
            b'&' => { res.extend(b"&amp;"); },
            _ => panic!("incorrect indexing"),
        }
    }
    if start < bytes.len() - 1 {
        res.extend(&bytes[start..]);
    }

    String::from_utf8(res).unwrap()
}

/// Alias for the `escape()` filter
pub fn e(s: &fmt::Display) -> String {
    escape(s)
}

/// Formats arguments according to the specified format
///
/// The first argument to this filter must be a string literal (as in normal
/// Rust). All arguments are passed through to the `format!()`
/// [macro](https://doc.rust-lang.org/stable/std/macro.format.html) by
/// the Askama code generator.
pub fn format() { }

/// Converts to lowercase.
pub fn lower(s: &fmt::Display) -> String {
    let s = format!("{}", s);
    s.to_lowercase()
}

/// Alias for the `lower()` filter.
pub fn lowercase(s: &fmt::Display) -> String {
    lower(s)
}

/// Converts to uppercase.
pub fn upper(s: &fmt::Display) -> String {
    let s = format!("{}", s);
    s.to_uppercase()
}

/// Alias for the `upper()` filter.
pub fn uppercase(s: &fmt::Display) -> String {
    upper(s)
}

/// Strip leading and trailing whitespace.
pub fn trim(s: &fmt::Display) -> String {
    let s = format!("{}", s);
    s.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape() {
        assert_eq!(escape(&""), "");
        assert_eq!(escape(&"<&>"), "&lt;&amp;&gt;");
        assert_eq!(escape(&"bla&"), "bla&amp;");
        assert_eq!(escape(&"<foo"), "&lt;foo");
    }

    #[test]
    fn test_lower() {
        assert_eq!(lower(&"Foo"), "foo");
        assert_eq!(lower(&"FOO"), "foo");
        assert_eq!(lower(&"FooBar"), "foobar");
        assert_eq!(lower(&"foo"), "foo");
    }

    #[test]
    fn test_upper() {
        assert_eq!(upper(&"Foo"), "FOO");
        assert_eq!(upper(&"FOO"), "FOO");
        assert_eq!(upper(&"FooBar"), "FOOBAR");
        assert_eq!(upper(&"foo"), "FOO");
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(&" Hello\tworld\t"), "Hello\tworld");
    }
}
