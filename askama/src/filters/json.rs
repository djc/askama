use std::convert::Infallible;
use std::{fmt, io, str};

use serde::Serialize;
use serde_json::to_writer_pretty;

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
pub fn json<S: Serialize>(s: S) -> Result<impl fmt::Display, Infallible> {
    Ok(ToJson(s))
}

#[derive(Debug, Clone)]
struct ToJson<S: Serialize>(S);

impl<S: Serialize> fmt::Display for ToJson<S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        to_writer_pretty(JsonWriter(f), &self.0).map_err(|_| fmt::Error)
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
    fn test_json() {
        assert_eq!(json(true).unwrap().to_string(), "true");
        assert_eq!(json("foo").unwrap().to_string(), r#""foo""#);
        assert_eq!(json(true).unwrap().to_string(), "true");
        assert_eq!(json("foo").unwrap().to_string(), r#""foo""#);
        assert_eq!(
            json("<script>").unwrap().to_string(),
            r#""\u003cscript\u003e""#
        );
        assert_eq!(
            json(vec!["foo", "bar"]).unwrap().to_string(),
            r#"[
  "foo",
  "bar"
]"#
        );
    }
}
