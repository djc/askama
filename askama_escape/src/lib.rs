#![no_std]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use core::fmt::{self, Display, Formatter, Write};
use core::str;

#[derive(Debug)]
pub struct MarkupDisplay<E, T>
where
    E: Escaper,
    T: Display,
{
    value: DisplayValue<T>,
    escaper: E,
}

impl<E, T> MarkupDisplay<E, T>
where
    E: Escaper,
    T: Display,
{
    pub fn new_unsafe(value: T, escaper: E) -> Self {
        Self {
            value: DisplayValue::Unsafe(value),
            escaper,
        }
    }

    pub fn new_safe(value: T, escaper: E) -> Self {
        Self {
            value: DisplayValue::Safe(value),
            escaper,
        }
    }

    #[must_use]
    pub fn mark_safe(mut self) -> MarkupDisplay<E, T> {
        self.value = match self.value {
            DisplayValue::Unsafe(t) => DisplayValue::Safe(t),
            _ => self.value,
        };
        self
    }
}

impl<E, T> Display for MarkupDisplay<E, T>
where
    E: Escaper,
    T: Display,
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self.value {
            DisplayValue::Unsafe(ref t) => write!(
                EscapeWriter {
                    fmt,
                    escaper: &self.escaper
                },
                "{t}"
            ),
            DisplayValue::Safe(ref t) => t.fmt(fmt),
        }
    }
}

#[derive(Debug)]
pub struct EscapeWriter<'a, E, W> {
    fmt: W,
    escaper: &'a E,
}

impl<E, W> Write for EscapeWriter<'_, E, W>
where
    W: Write,
    E: Escaper,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.escaper.write_escaped(&mut self.fmt, s)
    }
}

pub fn escape<E>(string: &str, escaper: E) -> Escaped<'_, E>
where
    E: Escaper,
{
    Escaped { string, escaper }
}

#[derive(Debug)]
pub struct Escaped<'a, E>
where
    E: Escaper,
{
    string: &'a str,
    escaper: E,
}

impl<E> Display for Escaped<'_, E>
where
    E: Escaper,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.escaper.write_escaped(fmt, self.string)
    }
}

pub struct Html;

impl Escaper for Html {
    fn write_escaped<W>(&self, mut fmt: W, string: &str) -> fmt::Result
    where
        W: Write,
    {
        let mut last = 0;
        for (index, byte) in string.bytes().enumerate() {
            const MIN_CHAR: u8 = b'"';
            const MAX_CHAR: u8 = b'>';
            const TABLE: [Option<&&str>; (MAX_CHAR - MIN_CHAR + 1) as usize] = {
                let mut table = [None; (MAX_CHAR - MIN_CHAR + 1) as usize];
                table[(b'<' - MIN_CHAR) as usize] = Some(&"&lt;");
                table[(b'>' - MIN_CHAR) as usize] = Some(&"&gt;");
                table[(b'&' - MIN_CHAR) as usize] = Some(&"&amp;");
                table[(b'"' - MIN_CHAR) as usize] = Some(&"&quot;");
                table[(b'\'' - MIN_CHAR) as usize] = Some(&"&#x27;");
                table
            };

            let escaped = match byte {
                MIN_CHAR..=MAX_CHAR => TABLE[(byte - MIN_CHAR) as usize],
                _ => None,
            };
            if let Some(escaped) = escaped {
                fmt.write_str(&string[last..index])?;
                fmt.write_str(escaped)?;
                last = index + 1;
            }
        }
        fmt.write_str(&string[last..])
    }
}

pub struct Text;

impl Escaper for Text {
    fn write_escaped<W>(&self, mut fmt: W, string: &str) -> fmt::Result
    where
        W: Write,
    {
        fmt.write_str(string)
    }
}

#[derive(Debug, PartialEq)]
enum DisplayValue<T>
where
    T: Display,
{
    Safe(T),
    Unsafe(T),
}

pub trait Escaper {
    fn write_escaped<W>(&self, fmt: W, string: &str) -> fmt::Result
    where
        W: Write;
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::string::ToString;

    #[test]
    fn test_escape() {
        assert_eq!(escape("", Html).to_string(), "");
        assert_eq!(escape("<&>", Html).to_string(), "&lt;&amp;&gt;");
        assert_eq!(escape("bla&", Html).to_string(), "bla&amp;");
        assert_eq!(escape("<foo", Html).to_string(), "&lt;foo");
        assert_eq!(escape("bla&h", Html).to_string(), "bla&amp;h");
    }
}
