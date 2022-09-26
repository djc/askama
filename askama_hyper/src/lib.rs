#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;
#[doc(hidden)]
pub use hyper;
use hyper::{header, Body, Response, StatusCode};

pub fn try_respond<T: Template>(t: &T) -> Result<Response<Body>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(T::MIME_TYPE),
        )
        .body(t.render()?.into())
        .map_err(|err| Error::Custom(Box::new(err)))
}

pub fn respond<T: Template>(t: &T) -> Response<Body> {
    match try_respond(t) {
        Ok(response) => response,
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap(),
    }
}
