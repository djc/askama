use std::fmt;

fn escapable(b: &u8) -> bool {
    *b == b'<' || *b == b'>' || *b == b'&'
}

pub fn escape(s: &fmt::Display) -> String {
    let s = format!("{}", s);
    let mut found = Vec::new();
    for (i, b) in s.as_bytes().iter().enumerate() {
        if escapable(b) {
            found.push(i);
        }
    }
    if found.is_empty() {
        return s;
    }

    let bytes = s.as_bytes();
    let max_len = bytes.len() + found.len() * 3;
    let mut res = Vec::<u8>::with_capacity(max_len);
    let mut start = 0;
    for idx in &found {
        if start < *idx {
            res.extend(&bytes[start..*idx]);
        }
        start = *idx + 1;
        match bytes[*idx] {
            b'<' => { res.extend(b"&lt;"); },
            b'>' => { res.extend(b"&gt;"); },
            b'&' => { res.extend(b"&amp;"); },
            _ => panic!("incorrect indexing"),
        }
    }
    if start < bytes.len() - 1 {
        res.extend(&bytes[start..]);
    }

    String::from_utf8(res).unwrap()
}

pub fn e(s: &fmt::Display) -> String {
    escape(s)
}

/// This is actually unused; format filter calls are forwarded directly to
/// the format!() macro in the code generator (see `visit_filter()`).
pub fn format() { }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_escape() {
        assert_eq!(escape(&""), "");
        assert_eq!(escape(&"<&>"), "&lt;&amp;&gt;");
        assert_eq!(escape(&"bla&"), "bla&amp;");
        assert_eq!(escape(&"<foo"), "&lt;foo");
    }
}
