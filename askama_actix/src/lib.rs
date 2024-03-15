#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::fmt;

#[doc(no_inline)]
pub use actix_web;
use actix_web::body::BoxBody;
use actix_web::http::header::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
#[doc(no_inline)]
pub use askama::*;

/// Render a [`Template`] into a [`HttpResponse`], or render an error page.
pub fn into_response<T: ?Sized + askama::Template>(tmpl: &T) -> HttpResponse<BoxBody> {
    try_into_response(tmpl).unwrap_or_else(|err| HttpResponse::from_error(ActixError(err)))
}

/// Try to render a [`Template`] into a [`HttpResponse`].
pub fn try_into_response<T: ?Sized + askama::Template>(
    tmpl: &T,
) -> Result<HttpResponse<BoxBody>, Error> {
    let value = tmpl.render()?;
    Ok(HttpResponseBuilder::new(StatusCode::OK)
        .content_type(HeaderValue::from_static(T::MIME_TYPE))
        .body(value))
}

/// Newtype to let askama::Error implement actix_web::ResponseError.
struct ActixError(Error);

impl fmt::Debug for ActixError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Error as fmt::Debug>::fmt(&self.0, f)
    }
}

impl fmt::Display for ActixError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Error as fmt::Display>::fmt(&self.0, f)
    }
}

impl ResponseError for ActixError {}
