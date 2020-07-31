pub use askama;
pub use tide;

use askama::*;
use tide::{http::Mime, Body, Response};

pub fn try_into_body<T: Template>(t: &T, ext: &str) -> Result<Body> {
    let string = t.render()?;
    let mut body = Body::from_string(string);

    if let Some(mime) = Mime::from_extension(ext) {
        body.set_mime(mime);
    }

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
