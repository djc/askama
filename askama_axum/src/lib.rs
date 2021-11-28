#![deny(elided_lifetimes_in_paths)]

pub use askama::*;
use axum::{
    self,
    body::{Bytes, Full},
    http::{Response, StatusCode},
};

pub fn into_response<T: Template>(t: &T, ext: &str) -> axum::http::Response<Full<Bytes>> {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                askama::mime::extension_to_mime_type(ext).to_string(),
            )
            .body(body.into())
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![].into())
            .unwrap(),
    }
}
