use crate::error::{Error, Result};
use askama_escape::{Escaper, MarkupDisplay};
use serde::Serialize;

/// Serialize to JSON (requires `serde_json` feature)
///
/// ## Errors
///
/// This will panic if `S`'s implementation of `Serialize` decides to fail,
/// or if `T` contains a map with non-string keys.
pub fn json<E: Escaper, S: Serialize>(e: E, s: S) -> Result<MarkupDisplay<E, String>> {
    match serde_json::to_string_pretty(&s) {
        Ok(s) => Ok(MarkupDisplay::new_safe(s, e)),
        Err(e) => Err(Error::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use askama_escape::Html;

    #[test]
    fn test_json() {
        assert_eq!(json(Html, true).unwrap().to_string(), "true");
        assert_eq!(json(Html, "foo").unwrap().to_string(), r#""foo""#);
        assert_eq!(json(Html, &true).unwrap().to_string(), "true");
        assert_eq!(json(Html, &"foo").unwrap().to_string(), r#""foo""#);
        assert_eq!(
            json(Html, &vec!["foo", "bar"]).unwrap().to_string(),
            r#"[
  "foo",
  "bar"
]"#
        );
    }
}
