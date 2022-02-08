#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

pub use askama::*;
use axum_core::body;
pub use axum_core::body::BoxBody;
pub use axum_core::response::IntoResponse;
pub use http::Response;
use http::StatusCode;
use http_body::{Empty, Full};

pub fn into_response<T: Template>(t: &T, _ext: &str) -> Response<BoxBody> {
    match t.render() {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static(T::MIME_TYPE),
            )
            .body(body::boxed(Full::from(body)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::boxed(Empty::new()))
            .unwrap(),
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_template {
    (($ident:ident) ($($impl_generics:tt)*) ($($orig_generics:tt)*) ($($where_clause:tt)*)) => {
        impl <$($impl_generics)*> $crate::IntoResponse
            for $ident <$($orig_generics)*> where $($where_clause)*
        {
            #[inline]
            fn into_response(self) -> $crate::Response<$crate::BoxBody> {
                $crate::into_response(&self, "")
            }
        }
    }
}
