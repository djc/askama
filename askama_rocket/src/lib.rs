use std::io::Cursor;

pub use askama::*;
use rocket::http::{ContentType, Status};
pub use rocket::request::Request;
use rocket::response::Response;
pub use rocket::response::{Responder, Result};

pub fn respond<'r, 'o: 'r, T: Template>(t: &'r T, ext: &'r str) -> Result<'o> {
    let rsp = t.render().map_err(|_| Status::InternalServerError)?;
    let ctype = ContentType::from_extension(ext).ok_or(Status::InternalServerError)?;
    Response::build()
        .header(ctype)
        .sized_body(rsp.len(), Cursor::new(rsp))
        .ok()
}