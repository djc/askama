#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::fmt;

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use askama::mime::extension_to_mime_type;
pub use askama::*;

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

pub trait TemplateToResponse {
    fn to_response(&self) -> HttpResponse<BoxBody>;
}

impl<T: askama::Template> TemplateToResponse for T {
    fn to_response(&self) -> HttpResponse<BoxBody> {
        match self.render() {
            Ok(buffer) => {
                let ctype = extension_to_mime_type(T::EXTENSION.unwrap_or("txt"));
                HttpResponseBuilder::new(StatusCode::OK)
                    .content_type(ctype)
                    .body(buffer)
            }
            Err(err) => HttpResponse::from_error(ActixError(err)),
        }
    }
}
