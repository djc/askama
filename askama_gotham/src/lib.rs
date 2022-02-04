#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;

pub use gotham::handler::IntoResponse;
use gotham::hyper::header;
pub use gotham::hyper::{Body, Response, StatusCode};
pub use gotham::state::State;

pub fn respond<T: Template>(t: &T, _ext: &str) -> Response<Body> {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(T::MIME_TYPE),
            )
            .body(body.into())
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(vec![].into())
            .unwrap(),
    }
}
