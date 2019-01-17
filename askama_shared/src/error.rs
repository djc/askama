use std::error::Error as ErrorTrait;
use std::fmt::{self, Display};

pub type Result<I> = ::std::result::Result<I, Error>;

/// askama error type
///
/// # Feature Interaction
///
/// If the feature `serde_json` is enabled an
/// additional error variant `Json` is added.
///
/// # Why not `failure`/`error-chain`?
///
/// Error from `error-chain` are not `Sync` which
/// can lead to problems e.g. when this is used
/// by a crate which use `failure`. Implementing
/// `Fail` on the other hand prevents the implementation
/// of `std::error::Error` until specialization lands
/// on stable. While errors impl. `Fail` can be
/// converted to a type impl. `std::error::Error`
/// using a adapter the benefits `failure` would
/// bring to this crate are small, which is why
/// `std::error::Error` was used.
///
#[derive(Debug)]
pub enum Error {
    /// formatting error
    Fmt(fmt::Error),

    /// json conversion error
    #[cfg(feature = "serde_json")]
    Json(::serde_json::Error),

    /// yaml conversion error
    #[cfg(feature = "serde_yaml")]
    Yaml(::serde_yaml::Error),

    /// This error needs to be non-exhaustive as
    /// the `Json` variants existence depends on
    /// a feature.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Fmt(ref err) => err.description(),
            #[cfg(feature = "serde_json")]
            Error::Json(ref err) => err.description(),
            _ => "unknown error: __Nonexhaustive",
        }
    }

    fn cause(&self) -> Option<&dyn ErrorTrait> {
        match *self {
            Error::Fmt(ref err) => err.source(),
            #[cfg(feature = "serde_json")]
            Error::Json(ref err) => err.source(),
            #[cfg(feature = "serde_yaml")]
            Error::Yaml(ref err) => err.source(),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Fmt(ref err) => write!(formatter, "formatting error: {}", err),
            #[cfg(feature = "serde_json")]
            Error::Json(ref err) => write!(formatter, "json conversion error: {}", err),
            #[cfg(feature = "serde_yaml")]
            Error::Yaml(ref err) => write!(formatter, "yaml conversion error: {}", err),
            _ => write!(formatter, "unknown error: __Nonexhaustive"),
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Self {
        Error::Fmt(err)
    }
}

#[cfg(feature = "serde_json")]
impl From<::serde_json::Error> for Error {
    fn from(err: ::serde_json::Error) -> Self {
        Error::Json(err)
    }
}

#[cfg(feature = "serde_yaml")]
impl From<::serde_yaml::Error> for Error {
    fn from(err: ::serde_yaml::Error) -> Self {
        Error::Yaml(err)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    trait AssertSendSyncStatic: Send + Sync + 'static {}
    impl AssertSendSyncStatic for Error {}
}
