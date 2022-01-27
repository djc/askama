use std::fmt::{self, Display};
use std::marker::PhantomData;

pub type Result<I, E = Error> = ::std::result::Result<I, E>;

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
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// formatting error
    Fmt(fmt::Error),

    Custom(Box<dyn std::error::Error + Send + Sync>),

    /// json conversion error
    #[cfg(feature = "serde_json")]
    Json(::serde_json::Error),

    /// yaml conversion error
    #[cfg(feature = "serde_yaml")]
    Yaml(::serde_yaml::Error),
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            Error::Fmt(ref err) => err.source(),
            Error::Custom(ref err) => Some(err.as_ref()),
            #[cfg(feature = "serde_json")]
            Error::Json(ref err) => err.source(),
            #[cfg(feature = "serde_yaml")]
            Error::Yaml(ref err) => err.source(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fmt(err) => write!(formatter, "formatting error: {}", err),
            Error::Custom(err) => write!(formatter, "{}", err),
            #[cfg(feature = "serde_json")]
            Error::Json(err) => write!(formatter, "json conversion error: {}", err),
            #[cfg(feature = "serde_yaml")]
            Error::Yaml(err) => write!(formatter, "yaml conversion error: {}", err),
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

#[doc(hidden)]
pub struct CustomErrorTag;

#[doc(hidden)]
pub struct CommonErrorTag;

#[doc(hidden)]
pub trait CustomErrorKind {
    #[inline]
    fn askama_error_kind(&self) -> CustomErrorTag {
        CustomErrorTag
    }
}

#[doc(hidden)]
pub trait CommonErrorKind {
    #[inline]
    fn askama_error_kind(&self) -> CommonErrorTag {
        CommonErrorTag
    }
}

impl CustomErrorTag {
    #[inline]
    pub fn convert(self, err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Error {
        Error::Custom(err.into())
    }
}

impl CommonErrorTag {
    #[inline]
    pub fn convert(self, error: impl Into<Error>) -> Error {
        error.into()
    }
}

#[doc(hidden)]
pub struct ErrorKindWrapper<E>(PhantomData<fn() -> *const E>);

#[doc(hidden)]
#[inline]
pub fn new_error_kind_wrapper<E>(err: E) -> (ErrorKindWrapper<E>, E) {
    let wrapper = ErrorKindWrapper(PhantomData);
    (wrapper, err)
}

impl<T: Into<Box<dyn std::error::Error + Send + Sync>>> CustomErrorKind for &ErrorKindWrapper<T> {}

impl<T: Into<Error>> CommonErrorKind for ErrorKindWrapper<T> {}

#[macro_export]
macro_rules! into_error {
    ($value:expr $(,)?) => {
        match $value {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(err) => {
                use ::askama::shared::error::{CommonErrorKind, CustomErrorKind};
                let (wrapper, err) = ::askama::shared::error::new_error_kind_wrapper(err);
                return ::askama::shared::Result::Err((&wrapper).askama_error_kind().convert(err));
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::Error;

    trait AssertSendSyncStatic: Send + Sync + 'static {}
    impl AssertSendSyncStatic for Error {}
}
