use std::convert::Infallible;
use std::{fmt, io, str};

use serde::Serialize;
use serde_json::ser::{to_writer, PrettyFormatter, Serializer};

/// Serialize to JSON (requires `json` feature)
///
/// The generated string does not contain ampersands `&`, chevrons `< >`, or apostrophes `'`.
/// To use it in a `<script>` you can combine it with the safe filter:
///
/// ``` html
/// <script>
/// var data = {{data|json|safe}};
/// </script>
/// ```
///
/// To use it in HTML attributes, you can either use it in quotation marks `"{{data|json}}"` as is,
/// or in apostrophes with the (optional) safe filter `'{{data|json|safe}}'`.
/// In HTML texts the output of e.g. `<pre>{{data|json|safe}}</pre>` is safe, too.
#[inline]
pub fn json(value: impl Serialize, indent: impl AsIndent) -> Result<impl fmt::Display, Infallible> {
    Ok(ToJson { value, indent })
}

pub trait AsIndent {
    fn as_indent(&self) -> Option<&str>;
}

impl AsIndent for str {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        Some(self)
    }
}

impl AsIndent for String {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        Some(self)
    }
}

impl AsIndent for isize {
    fn as_indent(&self) -> Option<&str> {
        const SPACES: &str = "                ";
        match *self < 0 {
            true => None,
            false => Some(&SPACES[..(*self as usize).min(SPACES.len())]),
        }
    }
}

impl AsIndent for () {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        None
    }
}

impl<T: AsIndent + ?Sized> AsIndent for &T {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        T::as_indent(self)
    }
}

impl<T: AsIndent> AsIndent for Option<T> {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        self.as_ref().and_then(T::as_indent)
    }
}

impl<T: AsIndent + ?Sized> AsIndent for Box<T> {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        T::as_indent(self.as_ref())
    }
}

impl<T: AsIndent + ToOwned + ?Sized> AsIndent for std::borrow::Cow<'_, T> {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        T::as_indent(self.as_ref())
    }
}

impl<T: AsIndent + ?Sized> AsIndent for std::rc::Rc<T> {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        T::as_indent(self.as_ref())
    }
}

impl<T: AsIndent + ?Sized> AsIndent for std::sync::Arc<T> {
    #[inline]
    fn as_indent(&self) -> Option<&str> {
        T::as_indent(self.as_ref())
    }
}

#[derive(Debug, Clone)]
struct ToJson<S, I> {
    value: S,
    indent: I,
}

impl<S: Serialize, I: AsIndent> fmt::Display for ToJson<S, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let f = JsonWriter(f);
        if let Some(indent) = self.indent.as_indent() {
            let formatter = PrettyFormatter::with_indent(indent.as_bytes());
            let mut serializer = Serializer::with_formatter(f, formatter);
            self.value
                .serialize(&mut serializer)
                .map_err(|_| fmt::Error)
        } else {
            to_writer(f, &self.value).map_err(|_| fmt::Error)
        }
    }
}

struct JsonWriter<'a, 'b: 'a>(&'a mut fmt::Formatter<'b>);

impl io::Write for JsonWriter<'_, '_> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.write_all(bytes)?;
        Ok(bytes.len())
    }

    #[inline]
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        write(self.0, bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn write(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    let mut last = 0;
    for (index, byte) in bytes.iter().enumerate() {
        let escaped = match byte {
            b'&' => Some(br"\u0026"),
            b'\'' => Some(br"\u0027"),
            b'<' => Some(br"\u003c"),
            b'>' => Some(br"\u003e"),
            _ => None,
        };
        if let Some(escaped) = escaped {
            f.write_str(unsafe { str::from_utf8_unchecked(&bytes[last..index]) })?;
            f.write_str(unsafe { str::from_utf8_unchecked(escaped) })?;
            last = index + 1;
        }
    }
    f.write_str(unsafe { str::from_utf8_unchecked(&bytes[last..]) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ugly() {
        assert_eq!(json(true, ()).unwrap().to_string(), "true");
        assert_eq!(json("foo", ()).unwrap().to_string(), r#""foo""#);
        assert_eq!(json(true, ()).unwrap().to_string(), "true");
        assert_eq!(json("foo", ()).unwrap().to_string(), r#""foo""#);
        assert_eq!(
            json("<script>", ()).unwrap().to_string(),
            r#""\u003cscript\u003e""#
        );
        assert_eq!(
            json(vec!["foo", "bar"], ()).unwrap().to_string(),
            r#"["foo","bar"]"#
        );
        assert_eq!(json(true, -1).unwrap().to_string(), "true");
        assert_eq!(json(true, Some(())).unwrap().to_string(), "true");
        assert_eq!(
            json(true, &Some(None::<isize>)).unwrap().to_string(),
            "true"
        );
    }

    #[test]
    fn test_pretty() {
        assert_eq!(json(true, "").unwrap().to_string(), "true");
        assert_eq!(
            json("<script>", Some("")).unwrap().to_string(),
            r#""\u003cscript\u003e""#
        );
        assert_eq!(
            json(vec!["foo", "bar"], Some("")).unwrap().to_string(),
            r#"[
"foo",
"bar"
]"#
        );
        assert_eq!(
            json(vec!["foo", "bar"], 2).unwrap().to_string(),
            r#"[
  "foo",
  "bar"
]"#
        );
        assert_eq!(
            json(vec!["foo", "bar"], &Some(&"————"))
                .unwrap()
                .to_string(),
            r#"[
————"foo",
————"bar"
]"#
        );
    }
}
