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

    if let Some(found) = found {
        let bytes = s.as_bytes();
        let mut res = Vec::with_capacity(s.len() + 6);
        res.extend(&bytes[0..found]);
        for c in bytes[found..].iter() {
            match *c {
                b'<' => {
                    res.extend(b"&lt;");
                }
                b'>' => {
                    res.extend(b"&gt;");
                }
                b'&' => {
                    res.extend(b"&amp;");
                }
                b'"' => {
                    res.extend(b"&quot;");
                }
                b'\'' => {
                    res.extend(b"&#x27;");
                }
                b'/' => {
                    res.extend(b"&#x2f;");
                }
                _ => res.push(*c),
            }
        }

        String::from_utf8(res).unwrap()
    } else {
        s
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
