use serde::Serialize;
use serde_json;
use errors::{Error, Result};

/// Serialize to JSON (requires `serde-json` feature)
///
/// ## Errors
///
/// This will panic if `S`'s implementation of `Serialize` decides to fail,
/// or if `T` contains a map with non-string keys.
pub fn json<S: Serialize>(s: &S) -> Result<String> {
    serde_json::to_string_pretty(s).map_err(Error::from)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        assert_eq!(json(&true).unwrap(), "true");
        assert_eq!(json(&"foo").unwrap(), r#""foo""#);
        assert_eq!(json(&vec!["foo", "bar"]).unwrap(),
r#"[
  "foo",
  "bar"
]"#);
    }
}
