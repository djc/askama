# Integrations

## Rocket integration

Enabling the `with-rocket` feature appends an implementation of Rocket's
`Responder` trait for each template type. This makes it easy to trivially
return a value of that type in a Rocket handler. See
[the example](https://github.com/djc/askama/blob/main/askama_rocket/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case a run-time error occurs during templating, a `500 Internal Server
Error` `Status` value will be returned, so that this can be further
handled by your error catcher.

## Actix-web integration

Enabling the `with-actix-web` feature appends an implementation of Actix-web's
`Responder` trait for each template type. This makes it easy to trivially return
a value of that type in an Actix-web handler. See
[the example](https://github.com/djc/askama/blob/main/askama_actix/tests/basic.rs)
from the Askama test suite for more on how to integrate.

## Axum integration

Enabling the `with-axum` feature appends an implementation of Axum's
`IntoResponse` trait for each template type. This makes it easy to trivially
return a value of that type in a Axum handler. See
[the example](https://github.com/djc/askama/blob/main/askama_axum/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case of a run-time error occurring during templating, the response will be of the same
signature, with a status code of `500 Internal Server Error`, mime `*/*`, and an empty `Body`.
This preserves the response chain if any custom error handling needs to occur.

## Gotham integration

Enabling the `with-gotham` feature appends an implementation of Gotham's
`IntoResponse` trait for each template type. This makes it easy to trivially
return a value of that type in a Gotham handler. See
[the example](https://github.com/djc/askama/blob/main/askama_gotham/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case of a run-time error occurring during templating, the response will be of the same
signature, with a status code of `500 Internal Server Error`, mime `*/*`, and an empty `Body`.
This preserves the response chain if any custom error handling needs to occur.

## Hyper integration

Enabling the `with-hyper` feature appends `From<Template> for hyper::Response<hyper::Body>` and
`TryFrom<Template> for hyper::Body` implementations for each template type. These
provide the ability for Hyper apps to build a response directly from
a template, or to create a templated hyper body to be used further in building a
`Response`. See [the example](https://github.com/djc/askama/blob/main/askama_hyper/tests/basic.rs)
from the Askama test suite for more on how to integrate.

When using `Template::from()` (or `Template::into()`),
the returned type is `hyper::Response<hyper::Body>`.
On the success of run-time rendering, the response consists of the status code `200`, the MIME type associated with the
template, and the body, created from the rendered String.
On the failure of run-time rendering, the response consists of the status code `500`, and an empty
body. The returned value can be seen being created in the integration's public `respond` function
in the
[integration's source](https://github.com/djc/askama/blob/main/askama_hyper/src/lib.rs).

When using `Template::try_from()` (or `Template::try_into()`),
the returned type is `Result<hyper::Body, askama::Error>`.
The generated code for the template does much less in this case, returning either a `hyper::Body`
when rendering was successful or the `askama::Error` when it was not. This is meant to allow the
caller more flexibility in how a response is built and in actions taken when rendering fails.

The
[integration's source](https://github.com/djc/askama/blob/main/askama_hyper/src/lib.rs)
also provides a public `try_respond` function that some may find useful to work with or work from.
It should not be confused with the `TryFrom` generated impl however; the generated impl
does not use either of the functions found in that source.

## Warp integration

Enabling the `with-warp` feature appends an implementation of Warp's `Reply`
trait for each template type. This makes it simple to return a template from
a Warp filter. See [the example](https://github.com/djc/askama/blob/main/askama_warp/tests/warp.rs)
from the Askama test suite for more on how to integrate.

## Tide integration

Enabling the `with-tide` feature appends `Into<tide::Response>` and
`TryInto<tide::Body>` implementations for each template type. This
provides the ability for tide apps to build a response directly from
a template, or to append a templated body to an existing
`Response`. See [the example](https://github.com/djc/askama/blob/main/askama_tide/tests/tide.rs)
from the Askama test suite for more on how to integrate.
