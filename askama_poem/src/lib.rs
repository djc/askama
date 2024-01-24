#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;
use poem::http::StatusCode;
pub use poem::{web::IntoResponse, Response};

pub fn into_response<T: Template>(t: &T) -> Response {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .content_type(T::MIME_TYPE)
            .body(body),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into(),
    }
}
