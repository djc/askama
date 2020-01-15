//! Module for built-in filter functions
//!
//! Contains all the built-in filter functions for use in templates.
//! You can define your own filters; for more information,
//! see the top-level crate documentation.
#![allow(clippy::trivially_copy_pass_by_ref)]

use std::fmt;

#[cfg(feature = "serde_json")]
mod json;
#[cfg(feature = "serde_json")]
pub use self::json::json;

#[cfg(feature = "serde_yaml")]
mod yaml;
#[cfg(feature = "serde_yaml")]
pub use self::yaml::yaml;

#[allow(unused_imports)]
use crate::error::Error::Fmt;
use askama_escape::{Escaper, MarkupDisplay};
#[cfg(feature = "humansize")]
use humansize::{file_size_opts, FileSize};
#[cfg(feature = "num_traits")]
use num_traits::{cast::NumCast, Signed};

use super::Result;

// This is used by the code generator to decide whether a named filter is part of
// Askama or should refer to a local `filters` module. It should contain all the
// filters shipped with Askama, even the optional ones (since optional inclusion
// in the const vector based on features seems impossible right now).
pub const BUILT_IN_FILTERS: [&str; 23] = [
    "abs",
    "capitalize",
    "center",
    "e",
    "escape",
    "filesizeformat",
    "format",
    "indent",
    "into_f64",
    "into_isize",
    "join",
    "linebreaks",
    "linebreaksbr",
    "lower",
    "lowercase",
    "safe",
    "trim",
    "truncate",
    "upper",
    "uppercase",
    "wordcount",
    "json", // Optional feature; reserve the name anyway
    "yaml", // Optional feature; reserve the name anyway
];

/// Marks a string (or other `Display` type) as safe
///
/// Use this is you want to allow markup in an expression, or if you know
/// that the expression's contents don't need to be escaped.
///
/// Askama will automatically insert the first (`Escaper`) argument,
/// so this filter only takes a single argument of any type that implements
/// `Display`.
pub fn safe<E, T>(e: E, v: T) -> Result<MarkupDisplay<E, T>>
where
    E: Escaper,
    T: fmt::Display,
{
    Ok(MarkupDisplay::new_safe(v, e))
}

/// Escapes `&`, `<` and `>` in strings
///
/// Askama will automatically insert the first (`Escaper`) argument,
/// so this filter only takes a single argument of any type that implements
/// `Display`.
pub fn escape<E, T>(e: E, v: T) -> Result<MarkupDisplay<E, T>>
where
    E: Escaper,
    T: fmt::Display,
{
    Ok(MarkupDisplay::new_unsafe(v, e))
}

/// Alias for the `escape()` filter
pub fn e<E, T>(e: E, v: T) -> Result<MarkupDisplay<E, T>>
where
    E: Escaper,
    T: fmt::Display,
{
    escape(e, v)
}

#[cfg(feature = "humansize")]
/// Returns adequate string representation (in KB, ..) of number of bytes
pub fn filesizeformat(b: usize) -> Result<String> {
    b.file_size(file_size_opts::DECIMAL)
        .map_err(|_| Fmt(fmt::Error))
}

/// Formats arguments according to the specified format
///
/// The first argument to this filter must be a string literal (as in normal
/// Rust). All arguments are passed through to the `format!()`
/// [macro](https://doc.rust-lang.org/stable/std/macro.format.html) by
/// the Askama code generator.
pub fn format() {}

/// Replaces line breaks in plain text with appropriate HTML
///
/// A single newline becomes an HTML line break `<br>` and a new line
/// followed by a blank line becomes a paragraph break `<p>`.
pub fn linebreaks(s: &dyn fmt::Display) -> Result<String> {
    let s = s.to_string();
    let linebroken = s.replace("\n\n", "</p><p>").replace("\n", "<br/>");

    Ok(format!("<p>{}</p>", linebroken))
}

/// Converts all newlines in a piece of plain text to HTML line breaks
pub fn linebreaksbr(s: &dyn fmt::Display) -> Result<String> {
    let s = s.to_string();
    Ok(s.replace("\n", "<br/>"))
}

/// Converts to lowercase
pub fn lower(s: &dyn fmt::Display) -> Result<String> {
    let s = s.to_string();
    Ok(s.to_lowercase())
}

/// Alias for the `lower()` filter
pub fn lowercase(s: &dyn fmt::Display) -> Result<String> {
    lower(s)
}

/// Converts to uppercase
pub fn upper(s: &dyn fmt::Display) -> Result<String> {
    let s = s.to_string();
    Ok(s.to_uppercase())
}

/// Alias for the `upper()` filter
pub fn uppercase(s: &dyn fmt::Display) -> Result<String> {
    upper(s)
}

/// Strip leading and trailing whitespace
pub fn trim(s: &dyn fmt::Display) -> Result<String> {
    let s = s.to_string();
    Ok(s.trim().to_owned())
}

/// Limit string length, appends '...' if truncated
pub fn truncate(s: &dyn fmt::Display, len: &usize) -> Result<String> {
    let mut s = s.to_string();
    if s.len() < *len {
        Ok(s)
    } else {
        s.truncate(*len);
        s.push_str("...");
        Ok(s)
    }
}

/// Indent lines with `width` spaces
pub fn indent(s: &dyn fmt::Display, width: &usize) -> Result<String> {
    let s = s.to_string();

    let mut indented = String::new();

    for (i, c) in s.char_indices() {
        indented.push(c);

        if c == '\n' && i < s.len() - 1 {
            for _ in 0..*width {
                indented.push(' ');
            }
        }
    }

    Ok(indented)
}

#[cfg(feature = "num_traits")]
/// Casts number to f64
pub fn into_f64<T>(number: T) -> Result<f64>
where
    T: NumCast,
{
    number.to_f64().ok_or(Fmt(fmt::Error))
}

#[cfg(feature = "num_traits")]
/// Casts number to isize
pub fn into_isize<T>(number: T) -> Result<isize>
where
    T: NumCast,
{
    number.to_isize().ok_or(Fmt(fmt::Error))
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

#[cfg(feature = "num_traits")]
/// Absolute value
pub fn abs<T>(number: T) -> Result<T>
where
    T: Signed,
{
    Ok(number.abs())
}

/// Capitalize a value. The first character will be uppercase, all others lowercase.
pub fn capitalize(s: &dyn fmt::Display) -> Result<String> {
    let mut s = s.to_string();

    match s.get_mut(0..1).map(|s| {
        s.make_ascii_uppercase();
        &*s
    }) {
        None => Ok(s),
        _ => match s.get_mut(1..).map(|s| {
            s.make_ascii_lowercase();
            &*s
        }) {
            _ => Ok(s),
        },
    }
}

/// Centers the value in a field of a given width
pub fn center(src: &dyn fmt::Display, dst_len: usize) -> Result<String> {
    let src = src.to_string();
    let len = src.len();

    if dst_len <= len {
        Ok(src)
    } else {
        let diff = dst_len - len;
        let mid = diff / 2;
        let r = diff % 2;
        let mut buf = String::with_capacity(dst_len);

        for _ in 0..mid {
            buf.push(' ');
        }

        buf.push_str(&src);

        for _ in 0..mid + r {
            buf.push(' ');
        }

        Ok(buf)
    }
}

/// Count the words in that string
pub fn wordcount(s: &dyn fmt::Display) -> Result<usize> {
    let s = s.to_string();

    Ok(s.split_whitespace().count())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "num_traits")]
    use std::f64::INFINITY;

    #[cfg(feature = "humansize")]
    #[test]
    fn test_filesizeformat() {
        assert_eq!(filesizeformat(0).unwrap(), "0 B");
        assert_eq!(filesizeformat(999).unwrap(), "999 B");
        assert_eq!(filesizeformat(1000).unwrap(), "1 KB");
        assert_eq!(filesizeformat(1023).unwrap(), "1.02 KB");
        assert_eq!(filesizeformat(1024).unwrap(), "1.02 KB");
    }

    #[test]
    fn test_linebreaks() {
        assert_eq!(
            linebreaks(&"Foo\nBar Baz").unwrap(),
            "<p>Foo<br/>Bar Baz</p>"
        );
        assert_eq!(
            linebreaks(&"Foo\nBar\n\nBaz").unwrap(),
            "<p>Foo<br/>Bar</p><p>Baz</p>"
        );
    }

    #[test]
    fn test_linebreaksbr() {
        assert_eq!(linebreaksbr(&"Foo\nBar").unwrap(), "Foo<br/>Bar");
        assert_eq!(
            linebreaksbr(&"Foo\nBar\n\nBaz").unwrap(),
            "Foo<br/>Bar<br/><br/>Baz"
        );
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

    #[test]
    fn test_indent() {
        assert_eq!(indent(&"hello", &2).unwrap(), "hello");
        assert_eq!(indent(&"hello\n", &2).unwrap(), "hello\n");
        assert_eq!(indent(&"hello\nfoo", &2).unwrap(), "hello\n  foo");
        assert_eq!(
            indent(&"hello\nfoo\n bar", &4).unwrap(),
            "hello\n    foo\n     bar"
        );
    }

    #[cfg(feature = "num_traits")]
    #[test]
    #[allow(clippy::float_cmp)]
    fn test_into_f64() {
        assert_eq!(into_f64(1).unwrap(), 1.0 as f64);
        assert_eq!(into_f64(1.9).unwrap(), 1.9 as f64);
        assert_eq!(into_f64(-1.9).unwrap(), -1.9 as f64);
        assert_eq!(into_f64(INFINITY as f32).unwrap(), INFINITY);
        assert_eq!(into_f64(-INFINITY as f32).unwrap(), -INFINITY);
    }

    #[cfg(feature = "num_traits")]
    #[test]
    fn test_into_isize() {
        assert_eq!(into_isize(1).unwrap(), 1 as isize);
        assert_eq!(into_isize(1.9).unwrap(), 1 as isize);
        assert_eq!(into_isize(-1.9).unwrap(), -1 as isize);
        assert_eq!(into_isize(1.5 as f64).unwrap(), 1 as isize);
        assert_eq!(into_isize(-1.5 as f64).unwrap(), -1 as isize);
        match into_isize(INFINITY) {
            Err(Fmt(fmt::Error)) => {}
            _ => panic!("Should return error of type Err(Fmt(fmt::Error))"),
        };
    }

    #[test]
    fn test_join() {
        assert_eq!(
            join((&["hello", "world"]).iter(), ", ").unwrap(),
            "hello, world"
        );
        assert_eq!(join((&["hello"]).iter(), ", ").unwrap(), "hello");

        let empty: &[&str] = &[];
        assert_eq!(join(empty.iter(), ", ").unwrap(), "");

        let input: Vec<String> = vec!["foo".into(), "bar".into(), "bazz".into()];
        assert_eq!(
            join((&input).iter(), ":".to_string()).unwrap(),
            "foo:bar:bazz"
        );
        assert_eq!(join(input.clone().iter(), ":").unwrap(), "foo:bar:bazz");
        assert_eq!(join(input.iter(), ":".to_string()).unwrap(), "foo:bar:bazz");

        let input: &[String] = &["foo".into(), "bar".into()];
        assert_eq!(join(input.iter(), ":").unwrap(), "foo:bar");
        assert_eq!(join(input.iter(), ":".to_string()).unwrap(), "foo:bar");

        let real: String = "blah".into();
        let input: Vec<&str> = vec![&real];
        assert_eq!(join(input.iter(), ";").unwrap(), "blah");

        assert_eq!(
            join((&&&&&["foo", "bar"]).iter(), ", ").unwrap(),
            "foo, bar"
        );
    }

    #[cfg(feature = "num_traits")]
    #[test]
    #[allow(clippy::float_cmp)]
    fn test_abs() {
        assert_eq!(abs(1).unwrap(), 1);
        assert_eq!(abs(-1).unwrap(), 1);
        assert_eq!(abs(1.0).unwrap(), 1.0);
        assert_eq!(abs(-1.0).unwrap(), 1.0);
        assert_eq!(abs(1.0 as f64).unwrap(), 1.0 as f64);
        assert_eq!(abs(-1.0 as f64).unwrap(), 1.0 as f64);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize(&"foo").unwrap(), "Foo".to_string());
        assert_eq!(capitalize(&"f").unwrap(), "F".to_string());
        assert_eq!(capitalize(&"fO").unwrap(), "Fo".to_string());
        assert_eq!(capitalize(&"").unwrap(), "".to_string());
        assert_eq!(capitalize(&"FoO").unwrap(), "Foo".to_string());
        assert_eq!(capitalize(&"foO BAR").unwrap(), "Foo bar".to_string());
    }

    #[test]
    fn test_center() {
        assert_eq!(center(&"f", 3).unwrap(), " f ".to_string());
        assert_eq!(center(&"f", 4).unwrap(), " f  ".to_string());
        assert_eq!(center(&"foo", 1).unwrap(), "foo".to_string());
        assert_eq!(center(&"foo bar", 8).unwrap(), "foo bar ".to_string());
    }

    #[test]
    fn test_wordcount() {
        assert_eq!(wordcount(&"").unwrap(), 0);
        assert_eq!(wordcount(&" \n\t").unwrap(), 0);
        assert_eq!(wordcount(&"foo").unwrap(), 1);
        assert_eq!(wordcount(&"foo bar").unwrap(), 2);
    }
}
