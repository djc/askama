#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;
pub use http::StatusCode;
pub use poem::{IntoResponse, Response};

pub fn into_response<T: Template>(t: &T) -> Response {
    match t.render() {
        Ok(body) => Response::builder()
            .header(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static(T::MIME_TYPE),
            )
            .body(body)
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
