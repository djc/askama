#![deny(elided_lifetimes_in_paths)]

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};
use askama::mime::extension_to_mime_type;
pub use askama::*;

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
            Err(err) => {
                HttpResponse::from_error(Box::new(err) as Box<dyn std::error::Error + 'static>)
            }
        }
    }
}
