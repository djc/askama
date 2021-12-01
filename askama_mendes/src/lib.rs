#![deny(elided_lifetimes_in_paths)]

use std::convert::TryFrom;

use mendes::application::{Application, Responder};
use mendes::http::header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE};
use mendes::http::request::Parts;
use mendes::http::Response;
use mime_guess::MimeGuess;

pub use askama::*;

pub fn into_response<A, T>(
    app: &A,
    req: &Parts,
    t: &T,
    ext: Option<&str>,
) -> Response<A::ResponseBody>
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

    let mut builder = Response::builder();
    builder = builder.header(CONTENT_LENGTH, content.len());
    if let Some(ext) = ext {
        if let Some(ty) = MimeGuess::from_ext(ext).first() {
            builder = builder.header(CONTENT_TYPE, HeaderValue::try_from(ty.as_ref()).unwrap());
        }
    }

    builder.body(content.into()).unwrap()
}
