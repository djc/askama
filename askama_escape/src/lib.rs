use std::fmt::{self, Display, Formatter};
use std::io::{self, prelude::*};
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
            MarkupDisplay::Unsafe(ref t) => {
                let mut w = EscapeWriter { fmt: f };
                write!(w, "{}", t).map_err(|_e| fmt::Error)
            }
            MarkupDisplay::Safe(ref t) => t.fmt(f),
        }
    }
}

pub struct EscapeWriter<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
}

impl io::Write for EscapeWriter<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        write_escaped_str(self.fmt, bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn escape(s: &str) -> Escaped<'_> {
    Escaped {
        bytes: s.as_bytes(),
    }
}

macro_rules! escaping_body {
    ($start:ident, $i:ident, $fmt:ident, $bytes:ident, $quote:expr) => {{
        if $start < $i {
            $fmt.write_str(unsafe { str::from_utf8_unchecked(&$bytes[$start..$i]) })?;
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
        write_escaped_str(fmt, self.bytes)
    }
}

fn write_escaped_str(fmt: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    let mut start = 0;
    for (i, b) in bytes.iter().enumerate() {
        if b.wrapping_sub(b'"') <= FLAG {
            match *b {
                b'<' => escaping_body!(start, i, fmt, bytes, "&lt;"),
                b'>' => escaping_body!(start, i, fmt, bytes, "&gt;"),
                b'&' => escaping_body!(start, i, fmt, bytes, "&amp;"),
                b'"' => escaping_body!(start, i, fmt, bytes, "&quot;"),
                b'\'' => escaping_body!(start, i, fmt, bytes, "&#x27;"),
                b'/' => escaping_body!(start, i, fmt, bytes, "&#x2f;"),
                _ => (),
            }
        }
    }
    fmt.write_str(unsafe { str::from_utf8_unchecked(&bytes[start..]) })?;
    Ok(())
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
