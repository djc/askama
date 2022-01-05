#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;

pub use gotham::handler::IntoResponse;
pub use gotham::state::State;
pub use hyper::{Body, Response, StatusCode};

pub fn respond<T: Template>(t: &T, ext: &str) -> Response<Body> {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                "content-type",
                mime::extension_to_mime_type(ext).to_string(),
            )
            .body(body.into())
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![].into())
            .unwrap(),
    }
}
