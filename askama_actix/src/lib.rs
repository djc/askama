#![deny(elided_lifetimes_in_paths)]

pub use askama::*;
use bytes::BytesMut;

use actix_web::{error::ErrorInternalServerError, HttpResponse};

pub trait TemplateToResponse {
    fn to_response(&self) -> HttpResponse;
}

impl<T: askama::Template> TemplateToResponse for T {
    fn to_response(&self) -> HttpResponse {
        let mut buffer = BytesMut::with_capacity(T::SIZE_HINT);
        if self.render_into(&mut buffer).is_err() {
            return ErrorInternalServerError("Template parsing error").error_response();
        }

        let ctype = askama::mime::extension_to_mime_type(T::EXTENSION.unwrap_or("txt")).to_string();
        HttpResponse::Ok()
            .content_type(ctype.as_str())
            .body(buffer.freeze())
    }
}

// Re-exported for use by generated code
#[doc(hidden)]
pub mod futures {
    pub use futures_util::future::ready;
    pub use futures_util::future::Ready;
}
