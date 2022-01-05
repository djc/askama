#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;
use axum_core::body;
pub use axum_core::body::BoxBody;
pub use axum_core::response::IntoResponse;
pub use http::Response;
use http::StatusCode;
use http_body::{Empty, Full};

pub fn into_response<T: Template>(t: &T, ext: &str) -> Response<BoxBody> {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                askama::mime::extension_to_mime_type(ext).to_string(),
            )
            .body(body::boxed(Full::from(body)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::boxed(Empty::new()))
            .unwrap(),
    }
}
