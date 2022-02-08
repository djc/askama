#![forbid(unsafe_code)]
#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]

#[doc(hidden)]
pub use mendes::application::{Application, IntoResponse};
use mendes::http::header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE};
#[doc(hidden)]
pub use mendes::http::request::Parts;
#[doc(hidden)]
pub use mendes::http::Response;

pub use askama::*;

pub fn into_response<A, T>(
    app: &A,
    req: &Parts,
    t: &T,
    _ext: Option<&str>,
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

    Response::builder()
        .header(CONTENT_LENGTH, content.len())
        .header(CONTENT_TYPE, HeaderValue::from_static(T::MIME_TYPE))
        .body(content.into())
        .unwrap()
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_template {
    (
        ($ident:ident)
        ($($impl_lifetimes:lifetime),* $(,)? $($first_type:ident $(impl_further:tt)*)?)
        ($($orig_generics:tt)*)
        ($($where_clause:tt)*)
    ) => {
        impl <
            $($impl_lifetimes,)*
            A: $crate::Application,
            $($first_type $(impl_further)*)?
        >
        $crate::IntoResponse<A> for $ident <$($orig_generics)*>
        where
            A::ResponseBody: ::std::convert::From<::std::string::String>,
            A::Error: ::std::convert::From<$crate::Error>,
            $($where_clause)*
        {
            #[inline]
            fn into_response(
                self,
                app: &A,
                req: &$crate::Parts,
            ) -> $crate::Response<A::ResponseBody> {
                $crate::into_response(app, req, &self, ::std::option::Option::None)
            }
        }
    }
}
