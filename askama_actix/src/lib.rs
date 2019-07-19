//! # Actix-web integration for Askama
//!
//! Contains an implementation of Actix-web's
//! `Responder` trait for each template type. This makes it easy to trivially return
//! a value of that type in an Actix-web handler. See
//! [the example](https://github.com/djc/askama/blob/master/testing/tests/actix_web.rs)
//! from the Askama test suite for more on how to integrate.

extern crate askama;
extern crate actix_web;
extern crate bytes;
extern crate mime_guess;

use std::fmt;

struct BytesWriter {
    buf: bytes::BytesMut,
}

impl BytesWriter {
    #[inline]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            buf: bytes::BytesMut::with_capacity(size),
        }
    }

    #[inline]
    pub fn freeze(self) -> bytes::Bytes {
        self.buf.freeze()
    }
}

impl fmt::Write for BytesWriter {
    #[inline]
    fn write_str(&mut self, buf: &str) -> fmt::Result {
        self.buf.extend_from_slice(buf.as_bytes());
        Ok(())
    }
}

// actix_web technically has this as a pub fn in later versions, fs::file_extension_to_mime.
// Older versions that don't have it exposed are easier this way. If ext is empty or no
// associated type was found, then this returns `application/octet-stream`, in line with how
// actix_web handles it in newer releases.
pub use self::actix_web::{
    error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
};

pub trait TemplateIntoResponse {
    fn into_response(&self) -> Result<HttpResponse, Error>;
}

impl<T: askama::Template> TemplateIntoResponse for T {
    fn into_response(&self) -> Result<HttpResponse, Error> {
        let mut buffer = BytesWriter::with_capacity(T::size_hint());
        self.render_into(&mut buffer)
            .map_err(|_| ErrorInternalServerError("Template parsing error"))?;

        let ctype = mime_guess::get_mime_type(T::extension().unwrap_or("txt")).to_string();
        Ok(HttpResponse::Ok()
            .content_type(ctype.as_str())
            .body(buffer.freeze()))
    }
}
