#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;
pub use warp;

use warp::http::{self, header, StatusCode};
use warp::hyper::Body;
use warp::reply::Response;

pub fn reply<T: askama::Template>(t: &T, _ext: &str) -> Response {
    match t.render() {
        Ok(body) => http::Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, T::MIME_TYPE)
            .body(body.into()),
        Err(_) => http::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty()),
    }
    .unwrap()
}
