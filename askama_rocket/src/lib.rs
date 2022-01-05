#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

use std::io::Cursor;

pub use askama::*;
use rocket::http::{ContentType, Status};
pub use rocket::request::Request;
use rocket::response::Response;
pub use rocket::response::{Responder, Result};

pub fn respond<T: Template>(t: &T, ext: &str) -> Result<'static> {
    let rsp = t.render().map_err(|_| Status::InternalServerError)?;
    let ctype = ContentType::from_extension(ext).ok_or(Status::InternalServerError)?;
    Response::build()
        .header(ctype)
        .sized_body(Cursor::new(rsp))
        .ok()
}
