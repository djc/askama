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

fn escapable(b: u8) -> bool {
    match b {
        b'<' | b'>' | b'&' | b'"' | b'\'' | b'/' => true,
        _ => false,
    }
}

pub fn escape(s: String) -> String {
    let mut found = Vec::new();
    for (i, b) in s.as_bytes().iter().enumerate() {
        if escapable(*b) {
            found.push(i);
        }
    }
    if found.is_empty() {
        return s;
    }

    let bytes = s.as_bytes();
    let max_len = bytes.len() + found.len() * 5;
    let mut res = Vec::<u8>::with_capacity(max_len);
    let mut start = 0;
    for idx in &found {
        if start < *idx {
            res.extend(&bytes[start..*idx]);
        }
        start = *idx + 1;
        match bytes[*idx] {
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
            _ => panic!("incorrect indexing"),
        }
    }
    if start < bytes.len() {
        res.extend(&bytes[start..]);
    }

    String::from_utf8(res).unwrap()
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
