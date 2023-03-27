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

Enabling the `with-axum` feature and depending on the `askama_axum` crate appends an implementation of Axum's
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
