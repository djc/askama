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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MarkupDisplay::Unsafe(ref t) => escape(&t.to_string()).fmt(f),
            MarkupDisplay::Safe(ref t) => t.fmt(f),
        }
    }
}

const FLAG: u8 = b'>' - b'"';

pub fn escape(s: &str) -> Escaped {
    Escaped {
        bytes: s.as_bytes(),
    }
}

pub struct Escaped<'a> {
    bytes: &'a [u8],
}

enum State {
    Empty,
    Unescaped(usize),
}

impl<'a> ::std::fmt::Display for Escaped<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::State::*;
        let mut state = Empty;
        for (i, b) in self.bytes.iter().enumerate() {
            let next = if b.wrapping_sub(b'"') <= FLAG {
                match *b {
                    b'<' => Some("&lt;"),
                    b'>' => Some("&gt;"),
                    b'&' => Some("&amp;"),
                    b'"' => Some("&quot;"),
                    b'\'' => Some("&#x27;"),
                    b'/' => Some("&#x2f;"),
                    _ => None,
                }
            } else {
                None
            };
            state = match (state, next) {
                (Empty, None) => Unescaped(i),
                (s @ Unescaped(_), None) => s,
                (Empty, Some(escaped)) => {
                    fmt.write_str(escaped)?;
                    Empty
                }
                (Unescaped(start), Some(escaped)) => {
                    fmt.write_str(unsafe { str::from_utf8_unchecked(&self.bytes[start..i]) })?;
                    fmt.write_str(escaped)?;
                    Empty
                }
            };
        }
        if let Unescaped(start) = state {
            fmt.write_str(unsafe { str::from_utf8_unchecked(&self.bytes[start..]) })?;
        }
        Ok(())
    }
}

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
