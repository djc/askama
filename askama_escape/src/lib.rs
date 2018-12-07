use std::fmt::{self, Display, Formatter};
use std::str;

#[derive(Debug, PartialEq)]
pub enum MarkupDisplay<T>
where
    T: Display,
{
    Safe(T),
    Unsafe(T),
}

impl<T> MarkupDisplay<T>
where
    T: Display,
{
    pub fn mark_safe(self) -> MarkupDisplay<T> {
        match self {
            MarkupDisplay::Unsafe(t) => MarkupDisplay::Safe(t),
            _ => self,
        }
    }
}

impl<T> From<T> for MarkupDisplay<T>
where
    T: Display,
{
    fn from(t: T) -> MarkupDisplay<T> {
        MarkupDisplay::Unsafe(t)
    }
}

impl<T> Display for MarkupDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            MarkupDisplay::Unsafe(ref t) => escape(&t.to_string()).fmt(f),
            MarkupDisplay::Safe(ref t) => t.fmt(f),
        }
    }
}

pub fn escape(s: &str) -> Escaped<'_> {
    Escaped {
        bytes: s.as_bytes(),
    }
}

macro_rules! escaping_body {
    ($start:ident, $i:ident, $fmt:ident, $_self:ident, $quote:expr) => {{
        if $start < $i {
            $fmt.write_str(unsafe { str::from_utf8_unchecked(&$_self.bytes[$start..$i]) })?;
        }
        $fmt.write_str($quote)?;
        $start = $i + 1;
    }};
}

pub struct Escaped<'a> {
    bytes: &'a [u8],
}

impl<'a> ::std::fmt::Display for Escaped<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut start = 0;
        for (i, b) in self.bytes.iter().enumerate() {
            if b.wrapping_sub(b'"') <= FLAG {
                match *b {
                    b'<' => escaping_body!(start, i, fmt, self, "&lt;"),
                    b'>' => escaping_body!(start, i, fmt, self, "&gt;"),
                    b'&' => escaping_body!(start, i, fmt, self, "&amp;"),
                    b'"' => escaping_body!(start, i, fmt, self, "&quot;"),
                    b'\'' => escaping_body!(start, i, fmt, self, "&#x27;"),
                    b'/' => escaping_body!(start, i, fmt, self, "&#x2f;"),
                    _ => (),
                }
            }
        }
        fmt.write_str(unsafe { str::from_utf8_unchecked(&self.bytes[start..]) })?;
        Ok(())
    }
}

const FLAG: u8 = b'>' - b'"';

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape() {
        assert_eq!(escape("").to_string(), "");
        assert_eq!(escape("<&>").to_string(), "&lt;&amp;&gt;");
        assert_eq!(escape("bla&").to_string(), "bla&amp;");
        assert_eq!(escape("<foo").to_string(), "&lt;foo");
        assert_eq!(escape("bla&h").to_string(), "bla&amp;h");
    }
}
