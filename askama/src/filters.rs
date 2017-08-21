//! Module for built-in filter functions
//!
//! Contains all the built-in filter functions for use in templates.
//! Currently, there is no way to define filters outside this module.
use std::fmt;
use super::Result;

fn escapable(b: &u8) -> bool {
    *b == b'<' || *b == b'>' || *b == b'&'
}

/// Escapes `&`, `<` and `>` in strings
pub fn escape(s: &fmt::Display) -> Result<String> {
    let s = format!("{}", s);
    let mut found = Vec::new();
    for (i, b) in s.as_bytes().iter().enumerate() {
        if escapable(b) {
            found.push(i);
        }
    }
    if found.is_empty() {
        return Ok(s);
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

    Ok(String::from_utf8(res).unwrap())
}

/// Alias for the `escape()` filter
pub fn e(s: &fmt::Display) -> Result<String> {
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
pub fn lower(s: &fmt::Display) -> Result<String> {
    let s = format!("{}", s);
    Ok(s.to_lowercase())
}

/// Alias for the `lower()` filter.
pub fn lowercase(s: &fmt::Display) -> Result<String> {
    lower(s)
}

/// Converts to uppercase.
pub fn upper(s: &fmt::Display) -> Result<String> {
    let s = format!("{}", s);
    Ok(s.to_uppercase())
}

/// Alias for the `upper()` filter.
pub fn uppercase(s: &fmt::Display) -> Result<String> {
    upper(s)
}

/// Strip leading and trailing whitespace.
pub fn trim(s: &fmt::Display) -> Result<String> {
    let s = format!("{}", s);
    Ok(s.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape() {
        assert_eq!(escape(&"").unwrap(), "");
        assert_eq!(escape(&"<&>").unwrap(), "&lt;&amp;&gt;");
        assert_eq!(escape(&"bla&").unwrap(), "bla&amp;");
        assert_eq!(escape(&"<foo").unwrap(), "&lt;foo");
    }

    #[test]
    fn test_lower() {
        assert_eq!(lower(&"Foo").unwrap(), "foo");
        assert_eq!(lower(&"FOO").unwrap(), "foo");
        assert_eq!(lower(&"FooBar").unwrap(), "foobar");
        assert_eq!(lower(&"foo").unwrap(), "foo");
    }

    #[test]
    fn test_upper() {
        assert_eq!(upper(&"Foo").unwrap(), "FOO");
        assert_eq!(upper(&"FOO").unwrap(), "FOO");
        assert_eq!(upper(&"FooBar").unwrap(), "FOOBAR");
        assert_eq!(upper(&"foo").unwrap(), "FOO");
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(&" Hello\tworld\t").unwrap(), "Hello\tworld");
    }
}
