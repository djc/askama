# Integrations

## Rocket integration

In your template definitions, replace `askama::Template` with
[`askama_rocket::Template`][askama_rocket].

Enabling the `with-rocket` feature appends an implementation of Rocket's
`Responder` trait for each template type. This makes it easy to trivially
return a value of that type in a Rocket handler. See
[the example](https://github.com/djc/askama/blob/main/askama_rocket/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case a run-time error occurs during templating, a `500 Internal Server
Error` `Status` value will be returned, so that this can be further
handled by your error catcher.

## Actix-web integration

In your template definitions, replace `askama::Template` with
[`askama_actix::Template`][askama_actix].

Enabling the `with-actix-web` feature appends an implementation of Actix-web's
`Responder` trait for each template type. This makes it easy to trivially return
a value of that type in an Actix-web handler. See
[the example](https://github.com/djc/askama/blob/main/askama_actix/tests/basic.rs)
from the Askama test suite for more on how to integrate.

## Axum integration

In your template definitions, replace `askama::Template` with
[`askama_axum::Template`][askama_axum].

Enabling the `with-axum` feature appends an implementation of Axum's
`IntoResponse` trait for each template type. This makes it easy to trivially
return a value of that type in a Axum handler. See
[the example](https://github.com/djc/askama/blob/main/askama_axum/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case of a run-time error occurring during templating, the response will be of the same
signature, with a status code of `500 Internal Server Error`, mime `*/*`, and an empty `Body`.
This preserves the response chain if any custom error handling needs to occur.

## Warp integration

In your template definitions, replace `askama::Template` with
[`askama_warp::Template`][askama_warp].

Enabling the `with-warp` feature appends an implementation of Warp's `Reply`
trait for each template type. This makes it simple to return a template from
a Warp filter. See [the example](https://github.com/djc/askama/blob/main/askama_warp/tests/warp.rs)
from the Askama test suite for more on how to integrate.

[askama_rocket]: https://docs.rs/askama_rocket
[askama_actix]: https://docs.rs/askama_actix
[askama_axum]: https://docs.rs/askama_axum
[askama_gotham]: https://docs.rs/askama_gotham
[askama_warp]: https://docs.rs/askama_warp
