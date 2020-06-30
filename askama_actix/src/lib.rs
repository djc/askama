pub use askama::*;
use bytes::BytesMut;

use actix_web::{error::ErrorInternalServerError, Error, HttpResponse};

pub trait TemplateIntoResponse {
    fn into_response(&self) -> ::std::result::Result<HttpResponse, Error>;
}

impl<T: askama::Template> TemplateIntoResponse for T {
    fn into_response(&self) -> ::std::result::Result<HttpResponse, Error> {
        let mut buffer = BytesMut::with_capacity(self.size_hint());
        self.render_into(&mut buffer)
            .map_err(|_| ErrorInternalServerError("Template parsing error"))?;

        let ctype =
            askama::mime::extension_to_mime_type(self.extension().unwrap_or("txt")).to_string();
        Ok(HttpResponse::Ok()
            .content_type(ctype.as_str())
            .body(buffer.freeze()))
    }
}

// Re-exported for use by generated code
#[doc(hidden)]
pub mod futures {
    pub use futures_util::future::ready;
    pub use futures_util::future::Ready;
}
