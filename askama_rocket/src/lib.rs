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
        .sized_body(Cursor::new(rsp))
        .ok()
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_template {
    (($ident:ident) ($($impl_generics:tt)*) ($($orig_generics:tt)*) ($($where_clause:tt)*)) => {
        impl <'askama, $($impl_generics)*> $crate::Responder<'askama>
            for $ident <$($orig_generics)*> where $($where_clause)*
        {
            #[inline]
            fn respond_to(self, _: &$crate::Request<'_>) -> $crate::Result<'askama> {
                $crate::respond(&self, "")
            }
        }
    }
}
