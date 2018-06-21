use error::{Error, Result};
use serde::Serialize;
use serde_json;
use MarkupDisplay;

/// Serialize to JSON (requires `serde-json` feature)
///
/// ## Errors
///
/// This will panic if `S`'s implementation of `Serialize` decides to fail,
/// or if `T` contains a map with non-string keys.
pub fn json<S: Serialize>(s: &S) -> Result<MarkupDisplay<String>> {
    match serde_json::to_string_pretty(s) {
        Ok(s) => Ok(MarkupDisplay::Safe(s)),
        Err(e) => Err(Error::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        assert_eq!(json(&true).unwrap().unsafe_string(), "true");
        assert_eq!(json(&"foo").unwrap().unsafe_string(), r#""foo""#);
        assert_eq!(
            json(&vec!["foo", "bar"]).unwrap().unsafe_string(),
            r#"[
  "foo",
  "bar"
]"#
        );
    }
}
