#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use mendes::application::{Application, IntoResponse};
use mendes::http::header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE};
use mendes::http::request::Parts;
use mendes::http::Response;

pub use askama::*;

pub fn into_response<A, T>(app: &A, req: &Parts, t: &T) -> Response<A::ResponseBody>
where
    A: Application,
    T: Template,
    A::ResponseBody: From<String>,
    A::Error: From<askama::Error>,
{
    let content = match t.render() {
        Ok(content) => content,
        Err(e) => return <A::Error as From<_>>::from(e).into_response(app, req),
    };

    Response::builder()
        .header(CONTENT_LENGTH, content.len())
        .header(CONTENT_TYPE, HeaderValue::from_static(T::MIME_TYPE))
        .body(content.into())
        .unwrap()
}
