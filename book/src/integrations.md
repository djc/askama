# Integrations

## Rocket integration

Enabling the `with-rocket` feature appends an implementation of Rocket's
`Responder` trait for each template type. This makes it easy to trivially
return a value of that type in a Rocket handler. See
[the example](https://github.com/djc/askama/blob/master/askama_rocket/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case a run-time error occurs during templating, a `500 Internal Server
Error` `Status` value will be returned, so that this can be further
handled by your error catcher.

## Iron integration

Enabling the `with-iron` feature appends an implementation of Iron's
`Modifier<Response>` trait for each template type. This makes it easy to
trivially return a value of that type in an Iron handler. See
[the example](https://github.com/djc/askama/blob/master/askama_iron/tests/basic.rs)
from the Askama test suite for more on how to integrate.

Note that Askama's generated `Modifier<Response>` implementation currently
unwraps any run-time errors from the template. If you have a better
suggestion, please [file an issue](https://github.com/djc/askama/issues/new).

## Actix-web integration

Enabling the `with-actix-web` feature appends an implementation of Actix-web's
`Responder` trait for each template type. This makes it easy to trivially return
a value of that type in an Actix-web handler. See
[the example](https://github.com/djc/askama/blob/master/askama_actix/tests/basic.rs)
from the Askama test suite for more on how to integrate.

## Gotham integration

Enabling the `with-gotham` feature appends an implementation of Gotham's
`IntoResponse` trait for each template type. This makes it easy to trivially
return a value of that type in a Gotham handler. See
[the example](https://github.com/djc/askama/blob/master/askama_gotham/tests/basic.rs)
from the Askama test suite for more on how to integrate.

In case of a run-time error occurring during templating, the response will be of the same
signature, with a status code of `500 Internal Server Error`, mime `*/*`, and an empty `Body`.
This preserves the response chain if any custom error handling needs to occur.

## Warp integration

Enabling the `with-warp` feature appends an implementation of Warp's `Reply`
trait for each template type. This makes it simple to return a template from
a Warp filter. See [the example](https://github.com/djc/askama/blob/master/askama_warp/tests/warp.rs)
from the Askama test suite for more on how to integrate.

## The `json` filter

Enabling the `serde-json` filter will enable the use of the `json` filter.
This will output formatted JSON for any value that implements the required
`Serialize` trait.

```
{
  "foo": "{{ foo }}",
  "bar": {{ bar|json }}
}
```

## The `yaml` filter

Enabling the `serde-yaml` filter will enable the use of the `yaml` filter.
This will output formatted JSON for any value that implements the required
`Serialize` trait.

```
{{ foo|yaml }}
```