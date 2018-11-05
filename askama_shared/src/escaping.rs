use std::fmt::{self, Display, Formatter};

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
    pub fn unsafe_string(&self) -> String {
        match *self {
            MarkupDisplay::Safe(ref t) | MarkupDisplay::Unsafe(ref t) => format!("{}", t),
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
            MarkupDisplay::Unsafe(_) => write!(f, "{}", escape(self.unsafe_string())),
            MarkupDisplay::Safe(ref t) => t.fmt(f),
        }
    }
}

macro_rules! escaping_body {
    ($n:expr, $res:ident, $bytes:ident, $start:ident, $quote:expr) => {{
        let next = $n;
        if $start < next {
            $res.extend(&$bytes[$start..next]);
        }
        $res.extend($quote);
        $start = next + 1;
    }};
}

const FLAG: u8 = b'>' - b'"';
pub fn escape(s: String) -> String {
    let mut found = None;
    for (i, b) in s.as_bytes().iter().enumerate() {
        if b.wrapping_sub(b'"') <= FLAG {
            match *b {
                b'<' | b'>' | b'&' | b'"' | b'\'' | b'/' => {
                    found = Some(i);
                    break;
                }
                _ => (),
            };
        }
    }

    match found {
        None => s,
        Some(found) => {
            let bytes = s.as_bytes();
            // Heuristic with a conservative estimate to save on intermediate allocations
            let mut res = Vec::with_capacity(s.len() + s.len() / 10);
            res.extend(&bytes[0..found]);

            let mut start = found;
            for (i, c) in bytes[found..].iter().enumerate() {
                match *c {
                    b'<' => escaping_body!(found + i, res, bytes, start, b"&lt;"),
                    b'>' => escaping_body!(found + i, res, bytes, start, b"&gt;"),
                    b'&' => escaping_body!(found + i, res, bytes, start, b"&amp;"),
                    b'"' => escaping_body!(found + i, res, bytes, start, b"&quot;"),
                    b'\'' => escaping_body!(found + i, res, bytes, start, b"&#x27;"),
                    b'/' => escaping_body!(found + i, res, bytes, start, b"&#x2f;"),
                    _ => (),
                }
            }
            res.extend(&bytes[start..]);

            String::from_utf8(res).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape() {
        assert_eq!(escape("".to_string()), "");
        assert_eq!(escape("<&>".to_string()), "&lt;&amp;&gt;");
        assert_eq!(escape("bla&".to_string()), "bla&amp;");
        assert_eq!(escape("<foo".to_string()), "&lt;foo");
        assert_eq!(escape("bla&h".to_string()), "bla&amp;h");
    }
}
