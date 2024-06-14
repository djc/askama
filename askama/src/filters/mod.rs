//! Module for built-in filter functions
//!
//! Contains all the built-in filter functions for use in templates.
//! You can define your own filters, as well.
//! For more information, read the [book](https://djc.github.io/askama/filters.html).
#![allow(clippy::trivially_copy_pass_by_ref)]

use std::cell::Cell;
use std::convert::Infallible;
use std::fmt::{self, Write};

#[cfg(feature = "serde-json")]
mod json;
#[cfg(feature = "serde-json")]
pub use self::json::json;

use askama_escape::{Escaper, MarkupDisplay};
#[cfg(feature = "humansize")]
use dep_humansize::{ISizeFormatter, ToF64, DECIMAL};
#[cfg(feature = "num-traits")]
use dep_num_traits::{cast::NumCast, Signed};
#[cfg(feature = "percent-encoding")]
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

use super::Result;
#[allow(unused_imports)]
use crate::error::Error::Fmt;

#[cfg(feature = "percent-encoding")]
// Urlencode char encoding set. Only the characters in the unreserved set don't
// have any special purpose in any part of a URI and can be safely left
// unencoded as specified in https://tools.ietf.org/html/rfc3986.html#section-2.3
const URLENCODE_STRICT_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'_')
    .remove(b'.')
    .remove(b'-')
    .remove(b'~');

#[cfg(feature = "percent-encoding")]
// Same as URLENCODE_STRICT_SET, but preserves forward slashes for encoding paths
const URLENCODE_SET: &AsciiSet = &URLENCODE_STRICT_SET.remove(b'/');

// MAX_LEN is maximum allowed length for filters.
const MAX_LEN: usize = 10_000;

/// Marks a string (or other `Display` type) as safe
///
/// Use this is you want to allow markup in an expression, or if you know
/// that the expression's contents don't need to be escaped.
///
/// Askama will automatically insert the first (`Escaper`) argument,
/// so this filter only takes a single argument of any type that implements
/// `Display`.
#[inline]
pub fn safe<E, T>(e: E, v: T) -> Result<MarkupDisplay<E, T>, Infallible>
where
    E: Escaper,
    T: fmt::Display,
{
    Ok(MarkupDisplay::new_safe(v, e))
}

/// Escapes strings according to the escape mode.
///
/// Askama will automatically insert the first (`Escaper`) argument,
/// so this filter only takes a single argument of any type that implements
/// `Display`.
///
/// It is possible to optionally specify an escaper other than the default for
/// the template's extension, like `{{ val|escape("txt") }}`.
#[inline]
pub fn escape<E, T>(e: E, v: T) -> Result<MarkupDisplay<E, T>, Infallible>
where
    E: Escaper,
    T: fmt::Display,
{
    Ok(MarkupDisplay::new_unsafe(v, e))
}

/// Alias for [`escape()`]
#[inline]
pub fn e<E, T>(e: E, v: T) -> Result<MarkupDisplay<E, T>, Infallible>
where
    E: Escaper,
    T: fmt::Display,
{
    escape(e, v)
}

#[cfg(feature = "humansize")]
/// Returns adequate string representation (in KB, ..) of number of bytes
///
/// ## Example
/// ```
/// # use askama::Template;
/// #[derive(Template)]
/// #[template(
///     source = "Filesize: {{ size_in_bytes|filesizeformat }}.",
///     ext = "html"
/// )]
/// struct Example {
///     size_in_bytes: u64,
/// }
///
/// let tmpl = Example { size_in_bytes: 1_234_567 };
/// assert_eq!(tmpl.to_string(),  "Filesize: 1.23 MB.");
/// ```
#[inline]
pub fn filesizeformat(b: &impl ToF64) -> Result<impl fmt::Display, Infallible> {
    Ok(FilesizeFormatFilter(b.to_f64()))
}

#[cfg(feature = "humansize")]
#[derive(Debug, Clone, Copy)]
struct FilesizeFormatFilter(f64);

#[cfg(feature = "humansize")]
impl fmt::Display for FilesizeFormatFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", ISizeFormatter::new(self.0, &DECIMAL)))
    }
}

#[cfg(feature = "percent-encoding")]
/// Percent-encodes the argument for safe use in URI; does not encode `/`.
///
/// This should be safe for all parts of URI (paths segments, query keys, query
/// values). In the rare case that the server can't deal with forward slashes in
/// the query string, use [`urlencode_strict`], which encodes them as well.
///
/// Encodes all characters except ASCII letters, digits, and `_.-~/`. In other
/// words, encodes all characters which are not in the unreserved set,
/// as specified by [RFC3986](https://tools.ietf.org/html/rfc3986#section-2.3),
/// with the exception of `/`.
///
/// ```none,ignore
/// <a href="/metro{{ "/stations/Château d'Eau"|urlencode }}">Station</a>
/// <a href="/page?text={{ "look, unicode/emojis ✨"|urlencode }}">Page</a>
/// ```
///
/// To encode `/` as well, see [`urlencode_strict`](./fn.urlencode_strict.html).
///
/// [`urlencode_strict`]: ./fn.urlencode_strict.html
#[inline]
pub fn urlencode<T: fmt::Display>(s: T) -> Result<impl fmt::Display, Infallible> {
    Ok(UrlencodeFilter(s, URLENCODE_SET))
}

#[cfg(feature = "percent-encoding")]
/// Percent-encodes the argument for safe use in URI; encodes `/`.
///
/// Use this filter for encoding query keys and values in the rare case that
/// the server can't process them unencoded.
///
/// Encodes all characters except ASCII letters, digits, and `_.-~`. In other
/// words, encodes all characters which are not in the unreserved set,
/// as specified by [RFC3986](https://tools.ietf.org/html/rfc3986#section-2.3).
///
/// ```none,ignore
/// <a href="/page?text={{ "look, unicode/emojis ✨"|urlencode_strict }}">Page</a>
/// ```
///
/// If you want to preserve `/`, see [`urlencode`](./fn.urlencode.html).
#[inline]
pub fn urlencode_strict<T: fmt::Display>(s: T) -> Result<impl fmt::Display, Infallible> {
    Ok(UrlencodeFilter(s, URLENCODE_STRICT_SET))
}

#[cfg(feature = "percent-encoding")]
struct UrlencodeFilter<T>(T, &'static AsciiSet);

#[cfg(feature = "percent-encoding")]
impl<T: fmt::Display> fmt::Display for UrlencodeFilter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Writer<'a, 'b>(&'a mut fmt::Formatter<'b>, &'static AsciiSet);

        impl fmt::Write for Writer<'_, '_> {
            #[inline]
            fn write_str(&mut self, s: &str) -> fmt::Result {
                write!(self.0, "{}", utf8_percent_encode(s, self.1))
            }
        }

        write!(Writer(f, self.1), "{}", self.0)
    }
}

/// Formats arguments according to the specified format
///
/// The *second* argument to this filter must be a string literal (as in normal
/// Rust). The two arguments are passed through to the `format!()`
/// [macro](https://doc.rust-lang.org/stable/std/macro.format.html) by
/// the Askama code generator, but the order is swapped to support filter
/// composition.
///
/// ```ignore
/// {{ value | fmt("{:?}") }}
/// ```
///
/// Compare with [format](./fn.format.html).
pub fn fmt() {}

/// Formats arguments according to the specified format
///
/// The first argument to this filter must be a string literal (as in normal
/// Rust). All arguments are passed through to the `format!()`
/// [macro](https://doc.rust-lang.org/stable/std/macro.format.html) by
/// the Askama code generator.
///
/// ```ignore
/// {{ "{:?}{:?}" | format(value, other_value) }}
/// ```
///
/// Compare with [fmt](./fn.fmt.html).
pub fn format() {}

/// Replaces line breaks in plain text with appropriate HTML
///
/// A single newline becomes an HTML line break `<br>` and a new line
/// followed by a blank line becomes a paragraph break `<p>`.
#[inline]
pub fn linebreaks(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    fn linebreaks(s: String) -> Result<String, Infallible> {
        let linebroken = s.replace("\n\n", "</p><p>").replace('\n', "<br/>");
        Ok(format!("<p>{linebroken}</p>"))
    }
    linebreaks(s.to_string())
}

/// Converts all newlines in a piece of plain text to HTML line breaks
#[inline]
pub fn linebreaksbr(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    fn linebreaksbr(s: String) -> Result<String, Infallible> {
        Ok(s.replace('\n', "<br/>"))
    }
    linebreaksbr(s.to_string())
}

/// Replaces only paragraph breaks in plain text with appropriate HTML
///
/// A new line followed by a blank line becomes a paragraph break `<p>`.
/// Paragraph tags only wrap content; empty paragraphs are removed.
/// No `<br/>` tags are added.
#[inline]
pub fn paragraphbreaks(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    fn paragraphbreaks(s: String) -> Result<String, Infallible> {
        let linebroken = s.replace("\n\n", "</p><p>").replace("<p></p>", "");
        Ok(format!("<p>{linebroken}</p>"))
    }
    paragraphbreaks(s.to_string())
}

/// Converts to lowercase
#[inline]
pub fn lower(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    fn lower(s: String) -> Result<String, Infallible> {
        Ok(s.to_lowercase())
    }
    lower(s.to_string())
}

/// Alias for the `lower()` filter
#[inline]
pub fn lowercase(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    lower(s)
}

/// Converts to uppercase
#[inline]
pub fn upper(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    fn upper(s: String) -> Result<String, Infallible> {
        Ok(s.to_uppercase())
    }
    upper(s.to_string())
}

/// Alias for the `upper()` filter
#[inline]
pub fn uppercase(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    upper(s)
}

/// Strip leading and trailing whitespace
pub fn trim<T: fmt::Display>(s: T) -> Result<impl fmt::Display> {
    struct Collector(String);

    impl fmt::Write for Collector {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            match self.0.is_empty() {
                true => self.0.write_str(s.trim_start()),
                false => self.0.write_str(s),
            }
        }
    }

    let mut collector = Collector(String::new());
    write!(collector, "{s}")?;
    let Collector(mut s) = collector;
    s.truncate(s.trim_end().len());
    Ok(s)
}

/// Limit string length, appends '...' if truncated
#[inline]
pub fn truncate<S: fmt::Display>(
    source: S,
    remaining: usize,
) -> Result<impl fmt::Display, Infallible> {
    Ok(TruncateFilter { source, remaining })
}

struct TruncateFilter<S> {
    source: S,
    remaining: usize,
}

impl<S: fmt::Display> fmt::Display for TruncateFilter<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Writer<'a, 'b> {
            dest: Option<&'a mut fmt::Formatter<'b>>,
            remaining: usize,
        }

        impl fmt::Write for Writer<'_, '_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let Some(dest) = &mut self.dest else {
                    return Ok(());
                };
                let mut rem = self.remaining;
                if rem >= s.len() {
                    dest.write_str(s)?;
                    self.remaining -= s.len();
                } else {
                    if rem > 0 {
                        while !s.is_char_boundary(rem) {
                            rem += 1;
                        }
                        if rem == s.len() {
                            // Don't write "..." if the char bound extends to the end of string.
                            self.remaining = 0;
                            return dest.write_str(s);
                        }
                        dest.write_str(&s[..rem])?;
                    }
                    dest.write_str("...")?;
                    self.dest = None;
                }
                Ok(())
            }

            #[inline]
            fn write_char(&mut self, c: char) -> fmt::Result {
                match self.dest.is_some() {
                    true => self.write_str(c.encode_utf8(&mut [0; 4])),
                    false => Ok(()),
                }
            }

            #[inline]
            fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
                match self.dest.is_some() {
                    true => fmt::write(self, args),
                    false => Ok(()),
                }
            }
        }

        let mut writer = Writer {
            dest: Some(f),
            remaining: self.remaining,
        };
        write!(writer, "{}", self.source)
    }
}

/// Indent lines with `width` spaces
#[inline]
pub fn indent(s: impl ToString, width: usize) -> Result<impl fmt::Display, Infallible> {
    fn indent(s: String, width: usize) -> Result<String, Infallible> {
        if width >= MAX_LEN {
            return Ok(s);
        }
        let mut indented = String::new();
        for (i, c) in s.char_indices() {
            indented.push(c);

            if c == '\n' && i < s.len() - 1 {
                for _ in 0..width {
                    indented.push(' ');
                }
            }
        }
        Ok(indented)
    }
    indent(s.to_string(), width)
}

#[cfg(feature = "num-traits")]
/// Casts number to f64
pub fn into_f64<T>(number: T) -> Result<f64>
where
    T: NumCast,
{
    number.to_f64().ok_or(Fmt(fmt::Error))
}

#[cfg(feature = "num-traits")]
/// Casts number to isize
pub fn into_isize<T>(number: T) -> Result<isize>
where
    T: NumCast,
{
    number.to_isize().ok_or(Fmt(fmt::Error))
}

/// Joins iterable into a string separated by provided argument
#[inline]
pub fn join<I, S>(input: I, separator: S) -> Result<impl fmt::Display, Infallible>
where
    I: IntoIterator,
    I::Item: fmt::Display,
    S: fmt::Display,
{
    Ok(JoinFilter(Cell::new(Some((input, separator)))))
}

/// Result of the filter [`join()`].
///
/// ## Note
///
/// This struct implements [`fmt::Display`], but only produces a string once.
/// Any subsequent call to `.to_string()` will result in an empty string, because the iterator is
/// already consumed.
//
// The filter contains a [`Cell`], so we can modify iterator inside a method that takes `self` by
// reference: [`fmt::Display::fmt()`] normally has the contract that it will produce the same result
// in multiple invocations for the same object. We break this contract, because have to consume the
// iterator, unless we want to enforce `I: Clone`, nor do we want to "memorize" the result of the
// joined data.
struct JoinFilter<I, S>(Cell<Option<(I, S)>>);

impl<I, S> fmt::Display for JoinFilter<I, S>
where
    I: IntoIterator,
    I::Item: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some((iter, separator)) = self.0.take() else {
            return Ok(());
        };
        for (idx, token) in iter.into_iter().enumerate() {
            match idx {
                0 => f.write_fmt(format_args!("{token}"))?,
                _ => f.write_fmt(format_args!("{separator}{token}"))?,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "num-traits")]
/// Absolute value
pub fn abs<T>(number: T) -> Result<T>
where
    T: Signed,
{
    Ok(number.abs())
}

/// Capitalize a value. The first character will be uppercase, all others lowercase.
#[inline]
pub fn capitalize(s: impl ToString) -> Result<impl fmt::Display, Infallible> {
    fn capitalize(s: String) -> Result<String, Infallible> {
        match s.chars().next() {
            Some(c) => {
                let mut replacement: String = c.to_uppercase().collect();
                replacement.push_str(&s[c.len_utf8()..].to_lowercase());
                Ok(replacement)
            }
            _ => Ok(s),
        }
    }
    capitalize(s.to_string())
}

/// Centers the value in a field of a given width
#[inline]
pub fn center(src: impl fmt::Display, width: usize) -> Result<impl fmt::Display, Infallible> {
    Ok(Center { src, width })
}

struct Center<T> {
    src: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Center<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.width < MAX_LEN {
            write!(f, "{: ^1$}", self.src, self.width)
        } else {
            write!(f, "{}", self.src)
        }
    }
}

/// Count the words in that string.
#[inline]
pub fn wordcount(s: impl ToString) -> Result<usize, Infallible> {
    fn wordcount(s: String) -> Result<usize, Infallible> {
        Ok(s.split_whitespace().count())
    }
    wordcount(s.to_string())
}

/// Return a title cased version of the value. Words will start with uppercase letters, all
/// remaining characters are lowercase.
#[inline]
pub fn title(s: impl ToString) -> Result<String, Infallible> {
    let s = s.to_string();
    let mut need_capitalization = true;

    // Sadly enough, we can't mutate a string when iterating over its chars, likely because it could
    // change the size of a char, "breaking" the char indices.
    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_whitespace() {
            output.push(c);
            need_capitalization = true;
        } else if need_capitalization {
            match c.is_uppercase() {
                true => output.push(c),
                false => output.extend(c.to_uppercase()),
            }
            need_capitalization = false;
        } else {
            match c.is_lowercase() {
                true => output.push(c),
                false => output.extend(c.to_lowercase()),
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "humansize")]
    #[test]
    fn test_filesizeformat() {
        assert_eq!(filesizeformat(&0).unwrap().to_string(), "0 B");
        assert_eq!(filesizeformat(&999u64).unwrap().to_string(), "999 B");
        assert_eq!(filesizeformat(&1000i32).unwrap().to_string(), "1 kB");
        assert_eq!(filesizeformat(&1023).unwrap().to_string(), "1.02 kB");
        assert_eq!(filesizeformat(&1024usize).unwrap().to_string(), "1.02 kB");
    }

    #[cfg(feature = "percent-encoding")]
    #[test]
    fn test_urlencoding() {
        // Unreserved (https://tools.ietf.org/html/rfc3986.html#section-2.3)
        // alpha / digit
        assert_eq!(urlencode("AZaz09").unwrap().to_string(), "AZaz09");
        assert_eq!(urlencode_strict("AZaz09").unwrap().to_string(), "AZaz09");
        // other
        assert_eq!(urlencode("_.-~").unwrap().to_string(), "_.-~");
        assert_eq!(urlencode_strict("_.-~").unwrap().to_string(), "_.-~");

        // Reserved (https://tools.ietf.org/html/rfc3986.html#section-2.2)
        // gen-delims
        assert_eq!(
            urlencode(":/?#[]@").unwrap().to_string(),
            "%3A/%3F%23%5B%5D%40"
        );
        assert_eq!(
            urlencode_strict(":/?#[]@").unwrap().to_string(),
            "%3A%2F%3F%23%5B%5D%40"
        );
        // sub-delims
        assert_eq!(
            urlencode("!$&'()*+,;=").unwrap().to_string(),
            "%21%24%26%27%28%29%2A%2B%2C%3B%3D"
        );
        assert_eq!(
            urlencode_strict("!$&'()*+,;=").unwrap().to_string(),
            "%21%24%26%27%28%29%2A%2B%2C%3B%3D"
        );

        // Other
        assert_eq!(
            urlencode("žŠďŤňĚáÉóŮ").unwrap().to_string(),
            "%C5%BE%C5%A0%C4%8F%C5%A4%C5%88%C4%9A%C3%A1%C3%89%C3%B3%C5%AE"
        );
        assert_eq!(
            urlencode_strict("žŠďŤňĚáÉóŮ").unwrap().to_string(),
            "%C5%BE%C5%A0%C4%8F%C5%A4%C5%88%C4%9A%C3%A1%C3%89%C3%B3%C5%AE"
        );

        // Ferris
        assert_eq!(urlencode("🦀").unwrap().to_string(), "%F0%9F%A6%80");
        assert_eq!(urlencode_strict("🦀").unwrap().to_string(), "%F0%9F%A6%80");
    }

    #[test]
    fn test_linebreaks() {
        assert_eq!(
            linebreaks("Foo\nBar Baz").unwrap().to_string(),
            "<p>Foo<br/>Bar Baz</p>"
        );
        assert_eq!(
            linebreaks("Foo\nBar\n\nBaz").unwrap().to_string(),
            "<p>Foo<br/>Bar</p><p>Baz</p>"
        );
    }

    #[test]
    fn test_linebreaksbr() {
        assert_eq!(linebreaksbr("Foo\nBar").unwrap().to_string(), "Foo<br/>Bar");
        assert_eq!(
            linebreaksbr("Foo\nBar\n\nBaz").unwrap().to_string(),
            "Foo<br/>Bar<br/><br/>Baz"
        );
    }

    #[test]
    fn test_paragraphbreaks() {
        assert_eq!(
            paragraphbreaks("Foo\nBar Baz").unwrap().to_string(),
            "<p>Foo\nBar Baz</p>"
        );
        assert_eq!(
            paragraphbreaks("Foo\nBar\n\nBaz").unwrap().to_string(),
            "<p>Foo\nBar</p><p>Baz</p>"
        );
        assert_eq!(
            paragraphbreaks("Foo\n\n\n\n\nBar\n\nBaz")
                .unwrap()
                .to_string(),
            "<p>Foo</p><p>\nBar</p><p>Baz</p>"
        );
    }

    #[test]
    fn test_lower() {
        assert_eq!(lower("Foo").unwrap().to_string(), "foo");
        assert_eq!(lower("FOO").unwrap().to_string(), "foo");
        assert_eq!(lower("FooBar").unwrap().to_string(), "foobar");
        assert_eq!(lower("foo").unwrap().to_string(), "foo");
    }

    #[test]
    fn test_upper() {
        assert_eq!(upper("Foo").unwrap().to_string(), "FOO");
        assert_eq!(upper("FOO").unwrap().to_string(), "FOO");
        assert_eq!(upper("FooBar").unwrap().to_string(), "FOOBAR");
        assert_eq!(upper("foo").unwrap().to_string(), "FOO");
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(" Hello\tworld\t").unwrap().to_string(), "Hello\tworld");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 2).unwrap().to_string(), "he...");
        let a = String::from("您好");
        assert_eq!(a.len(), 6);
        assert_eq!(String::from("您").len(), 3);
        assert_eq!(truncate("您好", 1).unwrap().to_string(), "您...");
        assert_eq!(truncate("您好", 2).unwrap().to_string(), "您...");
        assert_eq!(truncate("您好", 3).unwrap().to_string(), "您...");
        assert_eq!(truncate("您好", 4).unwrap().to_string(), "您好");
        assert_eq!(truncate("您好", 5).unwrap().to_string(), "您好");
        assert_eq!(truncate("您好", 6).unwrap().to_string(), "您好");
        assert_eq!(truncate("您好", 7).unwrap().to_string(), "您好");
        let s = String::from("🤚a🤚");
        assert_eq!(s.len(), 9);
        assert_eq!(String::from("🤚").len(), 4);
        assert_eq!(truncate("🤚a🤚", 1).unwrap().to_string(), "🤚...");
        assert_eq!(truncate("🤚a🤚", 2).unwrap().to_string(), "🤚...");
        assert_eq!(truncate("🤚a🤚", 3).unwrap().to_string(), "🤚...");
        assert_eq!(truncate("🤚a🤚", 4).unwrap().to_string(), "🤚...");
        assert_eq!(truncate("🤚a🤚", 5).unwrap().to_string(), "🤚a...");
        assert_eq!(truncate("🤚a🤚", 6).unwrap().to_string(), "🤚a🤚");
        assert_eq!(truncate("🤚a🤚", 6).unwrap().to_string(), "🤚a🤚");
        assert_eq!(truncate("🤚a🤚", 7).unwrap().to_string(), "🤚a🤚");
        assert_eq!(truncate("🤚a🤚", 8).unwrap().to_string(), "🤚a🤚");
        assert_eq!(truncate("🤚a🤚", 9).unwrap().to_string(), "🤚a🤚");
        assert_eq!(truncate("🤚a🤚", 10).unwrap().to_string(), "🤚a🤚");
    }

    #[test]
    fn test_indent() {
        assert_eq!(indent("hello", 2).unwrap().to_string(), "hello");
        assert_eq!(indent("hello\n", 2).unwrap().to_string(), "hello\n");
        assert_eq!(indent("hello\nfoo", 2).unwrap().to_string(), "hello\n  foo");
        assert_eq!(
            indent("hello\nfoo\n bar", 4).unwrap().to_string(),
            "hello\n    foo\n     bar"
        );
        assert_eq!(
            indent("hello", 267_332_238_858).unwrap().to_string(),
            "hello"
        );
    }

    #[cfg(feature = "num-traits")]
    #[test]
    #[allow(clippy::float_cmp)]
    fn test_into_f64() {
        assert_eq!(into_f64(1).unwrap(), 1.0_f64);
        assert_eq!(into_f64(1.9).unwrap(), 1.9_f64);
        assert_eq!(into_f64(-1.9).unwrap(), -1.9_f64);
        assert_eq!(into_f64(f32::INFINITY).unwrap(), f64::INFINITY);
        assert_eq!(into_f64(-f32::INFINITY).unwrap(), -f64::INFINITY);
    }

    #[cfg(feature = "num-traits")]
    #[test]
    fn test_into_isize() {
        assert_eq!(into_isize(1).unwrap(), 1_isize);
        assert_eq!(into_isize(1.9).unwrap(), 1_isize);
        assert_eq!(into_isize(-1.9).unwrap(), -1_isize);
        assert_eq!(into_isize(1.5_f64).unwrap(), 1_isize);
        assert_eq!(into_isize(-1.5_f64).unwrap(), -1_isize);
        match into_isize(f64::INFINITY) {
            Err(Fmt(fmt::Error)) => {}
            _ => panic!("Should return error of type Err(Fmt(fmt::Error))"),
        };
    }

    #[allow(clippy::needless_borrow)]
    #[test]
    fn test_join() {
        assert_eq!(
            join((&["hello", "world"]).iter(), ", ")
                .unwrap()
                .to_string(),
            "hello, world"
        );
        assert_eq!(
            join((&["hello"]).iter(), ", ").unwrap().to_string(),
            "hello"
        );

        let empty: &[&str] = &[];
        assert_eq!(join(empty.iter(), ", ").unwrap().to_string(), "");

        let input: Vec<String> = vec!["foo".into(), "bar".into(), "bazz".into()];
        assert_eq!(join(input.iter(), ":").unwrap().to_string(), "foo:bar:bazz");

        let input: &[String] = &["foo".into(), "bar".into()];
        assert_eq!(join(input.iter(), ":").unwrap().to_string(), "foo:bar");

        let real: String = "blah".into();
        let input: Vec<&str> = vec![&real];
        assert_eq!(join(input.iter(), ";").unwrap().to_string(), "blah");

        assert_eq!(
            join((&&&&&["foo", "bar"]).iter(), ", ")
                .unwrap()
                .to_string(),
            "foo, bar"
        );
    }

    #[cfg(feature = "num-traits")]
    #[test]
    #[allow(clippy::float_cmp)]
    fn test_abs() {
        assert_eq!(abs(1).unwrap(), 1);
        assert_eq!(abs(-1).unwrap(), 1);
        assert_eq!(abs(1.0).unwrap(), 1.0);
        assert_eq!(abs(-1.0).unwrap(), 1.0);
        assert_eq!(abs(1.0_f64).unwrap(), 1.0_f64);
        assert_eq!(abs(-1.0_f64).unwrap(), 1.0_f64);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("foo").unwrap().to_string(), "Foo".to_string());
        assert_eq!(capitalize("f").unwrap().to_string(), "F".to_string());
        assert_eq!(capitalize("fO").unwrap().to_string(), "Fo".to_string());
        assert_eq!(capitalize("").unwrap().to_string(), "".to_string());
        assert_eq!(capitalize("FoO").unwrap().to_string(), "Foo".to_string());
        assert_eq!(
            capitalize("foO BAR").unwrap().to_string(),
            "Foo bar".to_string()
        );
        assert_eq!(
            capitalize("äØÄÅÖ").unwrap().to_string(),
            "Äøäåö".to_string()
        );
        assert_eq!(capitalize("ß").unwrap().to_string(), "SS".to_string());
        assert_eq!(capitalize("ßß").unwrap().to_string(), "SSß".to_string());
    }

    #[test]
    fn test_center() {
        assert_eq!(center("f", 3).unwrap().to_string(), " f ".to_string());
        assert_eq!(center("f", 4).unwrap().to_string(), " f  ".to_string());
        assert_eq!(center("foo", 1).unwrap().to_string(), "foo".to_string());
        assert_eq!(
            center("foo bar", 8).unwrap().to_string(),
            "foo bar ".to_string()
        );
        assert_eq!(
            center("foo", 111_669_149_696).unwrap().to_string(),
            "foo".to_string()
        );
    }

    #[test]
    fn test_wordcount() {
        assert_eq!(wordcount("").unwrap(), 0);
        assert_eq!(wordcount(" \n\t").unwrap(), 0);
        assert_eq!(wordcount("foo").unwrap(), 1);
        assert_eq!(wordcount("foo bar").unwrap(), 2);
        assert_eq!(wordcount("foo  bar").unwrap(), 2);
    }

    #[test]
    fn test_title() {
        assert_eq!(&title("").unwrap(), "");
        assert_eq!(&title(" \n\t").unwrap(), " \n\t");
        assert_eq!(&title("foo").unwrap(), "Foo");
        assert_eq!(&title(" foo").unwrap(), " Foo");
        assert_eq!(&title("foo bar").unwrap(), "Foo Bar");
        assert_eq!(&title("foo  bar ").unwrap(), "Foo  Bar ");
        assert_eq!(&title("fOO").unwrap(), "Foo");
        assert_eq!(&title("fOo BaR").unwrap(), "Foo Bar");
    }
}
