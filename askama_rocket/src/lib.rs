#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::io::Cursor;

pub use askama::*;
use rocket::http::{Header, Status};
pub use rocket::request::Request;
use rocket::response::Response;
pub use rocket::response::{Responder, Result};

pub fn respond<T: Template>(t: &T, _ext: &str) -> Result<'static> {
    let rsp = t.render().map_err(|_| Status::InternalServerError)?;
    Response::build()
        .header(Header::new("content-type", T::MIME_TYPE))
        .sized_body(rsp.len(), Cursor::new(rsp))
        .ok()
}
