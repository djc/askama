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

pub fn escape(s: &str) -> Escaped {
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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
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
        let escapes = "<>&\"'/";
        let escaped = "&lt;&gt;&amp;&quot;&#x27;&#x2f;";
        let string_long: &str = &"foobar".repeat(1024);

        assert_eq!(escape("").to_string(), "");
        assert_eq!(escape("<&>").to_string(), "&lt;&amp;&gt;");
        assert_eq!(escape("bar&").to_string(), "bar&amp;");
        assert_eq!(escape("<foo").to_string(), "&lt;foo");
        assert_eq!(escape("bar&h").to_string(), "bar&amp;h");
        assert_eq!(
            escape("// my <html> is \"unsafe\" & should be 'escaped'").to_string(),
            "&#x2f;&#x2f; my &lt;html&gt; is &quot;unsafe&quot; &amp; \
             should be &#x27;escaped&#x27;"
        );
        assert_eq!(escape(&"<".repeat(16)).to_string(), "&lt;".repeat(16));
        assert_eq!(escape(&"<".repeat(32)).to_string(), "&lt;".repeat(32));
        assert_eq!(escape(&"<".repeat(64)).to_string(), "&lt;".repeat(64));
        assert_eq!(escape(&"<".repeat(128)).to_string(), "&lt;".repeat(128));
        assert_eq!(escape(&"<".repeat(1024)).to_string(), "&lt;".repeat(1024));
        assert_eq!(escape(&"<".repeat(129)).to_string(), "&lt;".repeat(129));
        assert_eq!(
            escape(&"<".repeat(128 * 2 - 1)).to_string(),
            "&lt;".repeat(128 * 2 - 1)
        );
        assert_eq!(
            escape(&"<".repeat(128 * 8 - 1)).to_string(),
            "&lt;".repeat(128 * 8 - 1)
        );
        assert_eq!(escape(string_long).to_string(), string_long);
        assert_eq!(
            escape(&[string_long, "<"].join("")).to_string(),
            [string_long, "&lt;"].join("")
        );
        assert_eq!(
            escape(&["<", string_long].join("")).to_string(),
            ["&lt;", string_long].join("")
        );
        assert_eq!(
            escape(&escapes.repeat(1024)).to_string(),
            escaped.repeat(1024)
        );
        assert_eq!(
            escape(&[string_long, "<", string_long].join("")).to_string(),
            [string_long, "&lt;", string_long].join("")
        );
        assert_eq!(
            escape(&[string_long, "<", string_long, escapes, string_long,].join("")).to_string(),
            [string_long, "&lt;", string_long, escaped, string_long,].join("")
        );
    }
}
