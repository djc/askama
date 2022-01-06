#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama;
pub use tide;

use askama::*;
use tide::{Body, Response};

pub fn try_into_body<T: Template>(t: &T, _ext: &str) -> Result<Body> {
    let string = t.render()?;
    let mut body = Body::from_string(string);
    body.set_mime(T::MIME_TYPE);
    Ok(body)
}

pub fn into_response<T: Template>(t: &T, ext: &str) -> Response {
    match try_into_body(t, ext) {
        Ok(body) => {
            let mut response = Response::new(200);
            response.set_body(body);
            response
        }

        Err(error) => {
            let mut response = Response::new(500);
            response.set_error(error);
            response
        }
    }
}
