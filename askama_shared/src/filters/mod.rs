//! Module for built-in filter functions
//!
//! Contains all the built-in filter functions for use in templates.
//! Currently, there is no way to define filters outside this module.

#[cfg(feature = "serde-json")]
mod json;

#[cfg(feature = "serde-json")]
pub use self::json::json;

use std::fmt;

use super::Result;
use escaping::{self, MarkupDisplay};

// This is used by the code generator to decide whether a named filter is part of
// Askama or should refer to a local `filters` module. It should contain all the
// filters shipped with Askama, even the optional ones (since optional inclusion
// in the const vector based on features seems impossible right now).
pub const BUILT_IN_FILTERS: [&str; 12] = [
    "e",
    "escape",
    "format",
    "join",
    "lower",
    "lowercase",
    "safe",
    "trim",
    "truncate",
    "upper",
    "uppercase",
    "json", // Optional feature; reserve the name anyway
];

pub fn safe<D, I>(v: I) -> Result<MarkupDisplay<D>>
where
    D: fmt::Display,
    MarkupDisplay<D>: From<I>,
{
    let res: MarkupDisplay<D> = v.into();
    Ok(res.mark_safe())
}

/// Escapes `&`, `<` and `>` in strings
pub fn escape<D, I>(i: I) -> Result<MarkupDisplay<String>>
where
    D: fmt::Display,
    MarkupDisplay<D>: From<I>,
{
    let md: MarkupDisplay<D> = i.into();
    Ok(MarkupDisplay::Safe(escaping::escape(md.unsafe_string())))
}

/// Alias for the `escape()` filter
pub fn e<D, I>(i: I) -> Result<MarkupDisplay<String>>
where
    D: fmt::Display,
    MarkupDisplay<D>: From<I>,
{
    escape(i)
}

/// Formats arguments according to the specified format
///
/// The first argument to this filter must be a string literal (as in normal
/// Rust). All arguments are passed through to the `format!()`
/// [macro](https://doc.rust-lang.org/stable/std/macro.format.html) by
/// the Askama code generator.
pub fn format() {}

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

/// Limit string length, appens '...' if truncated {
pub fn truncate(s: &fmt::Display, len: &usize) -> Result<String> {
    let mut s = format!("{}", s);
    if s.len() < *len {
        Ok(s)
    } else {
        s.truncate(*len);
        s.push_str("...");
        Ok(s)
    }
}

/// Joins iterable into a string separated by provided argument
pub fn join<T, I, S>(input: I, separator: S) -> Result<String>
where
    T: fmt::Display,
    I: Iterator<Item = T>,
    S: AsRef<str>,
{
    let separator: &str = separator.as_ref();

    let mut rv = String::new();

    for (num, item) in input.enumerate() {
        if num > 0 {
            rv.push_str(separator);
        }

        rv.push_str(&format!("{}", item));
    }

    Ok(rv)
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_join() {
        assert_eq!(
            join((&["hello", "world"]).into_iter(), ", ").unwrap(),
            "hello, world"
        );
        assert_eq!(join((&["hello"]).into_iter(), ", ").unwrap(), "hello");

        let empty: &[&str] = &[];
        assert_eq!(join(empty.into_iter(), ", ").unwrap(), "");

        let input: Vec<String> = vec!["foo".into(), "bar".into(), "bazz".into()];
        assert_eq!(
            join((&input).into_iter(), ":".to_string()).unwrap(),
            "foo:bar:bazz"
        );
        assert_eq!(
            join(input.clone().into_iter(), ":").unwrap(),
            "foo:bar:bazz"
        );
        assert_eq!(
            join(input.clone().into_iter(), ":".to_string()).unwrap(),
            "foo:bar:bazz"
        );

        let input: &[String] = &["foo".into(), "bar".into()];
        assert_eq!(join(input.into_iter(), ":").unwrap(), "foo:bar");
        assert_eq!(join(input.into_iter(), ":".to_string()).unwrap(), "foo:bar");

        let real: String = "blah".into();
        let input: Vec<&str> = vec![&real];
        assert_eq!(join(input.into_iter(), ";").unwrap(), "blah");

        assert_eq!(
            join((&&&&&["foo", "bar"]).into_iter(), ", ").unwrap(),
            "foo, bar"
        );
    }
}
