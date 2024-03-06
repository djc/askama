#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

#[doc(no_inline)]
pub use askama::*;
#[doc(no_inline)]
pub use axum_core;
use axum_core::response::{IntoResponse, Response};

/// Render a [`Template`] into a [`Response`], or render an error page.
pub fn into_response<T: ?Sized + askama::Template>(tmpl: &T) -> Response {
    try_into_response(tmpl)
        .map_err(|err| axum_core::response::ErrorResponse::from(err.to_string()))
        .into_response()
}

/// Try to render a [`Template`] into a [`Response`].
pub fn try_into_response<T: ?Sized + askama::Template>(tmpl: &T) -> Result<Response, Error> {
    let value = tmpl.render()?.into();
    Response::builder()
        .header(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static(T::MIME_TYPE),
        )
        .body(value)
        .map_err(|err| Error::Custom(err.into()))
}
