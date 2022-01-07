use crate::CompileError;

use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

/// Whitespace handling of the input source.
///
/// The automatic stripping does not try to understand the input at all. It
/// does not know what `xml:space="preserve"`, `<pre>` or `<textarea>`
/// means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strip {
    /// "none": Don't strip any spaces in the input
    None,
    /// "tail": Remove a single single newline at the end of the input. This is the default
    Tail,
    /// "trim-lines": Remove all whitespaces at the front and back of all lines, and remove empty
    /// lines
    TrimLines,
    /// "eager": Like "trim", but also replace runs of whitespaces with a single space.
    Eager,
}

impl Default for Strip {
    fn default() -> Self {
        Strip::Tail
    }
}

impl TryFrom<&str> for Strip {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "none" => Ok(Strip::None),
            "tail" => Ok(Strip::Tail),
            "trim-lines" => Ok(Strip::TrimLines),
            "eager" => Ok(Strip::Eager),
            v => return Err(format!("invalid value for strip: {:?}", v)),
        }
    }
}

impl FromStr for Strip {
    type Err = CompileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into().map_err(Into::into)
    }
}

#[cfg(feature = "serde")]
const _: () = {
    use std::fmt;

    struct StripVisitor;

    impl<'de> serde::de::Visitor<'de> for StripVisitor {
        type Value = Strip;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, r#"the string "none", "tail", "trim-lines", or "eager""#)
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            s.try_into().map_err(E::custom)
        }
    }

    impl<'de> serde::Deserialize<'de> for Strip {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(StripVisitor)
        }
    }
};

impl Strip {
    pub fn apply<'a, S: Into<Cow<'a, str>>>(self, src: S) -> String {
        let src = src.into();
        match self {
            Strip::None => src.into_owned(),
            Strip::Tail => {
                let mut s = src.into_owned();
                if s.ends_with('\n') {
                    s.pop();
                }
                s
            }
            Strip::TrimLines | Strip::Eager => {
                let mut stripped = String::with_capacity(src.len());
                for line in src.lines().map(|s| s.trim()).filter(|&s| !s.is_empty()) {
                    if !stripped.is_empty() {
                        stripped.push('\n');
                    }
                    if self == Strip::Eager {
                        for (index, word) in line.split_ascii_whitespace().enumerate() {
                            if index > 0 {
                                stripped.push(' ');
                            }
                            stripped.push_str(word);
                        }
                    } else {
                        stripped.push_str(line);
                    }
                }
                stripped
            }
        }
    }
}
