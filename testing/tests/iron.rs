#[macro_use]
extern crate askama;
extern crate iron;

use askama::Template;
use iron::{status, Response};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_iron() {
    let rsp = Response::with((status::Ok, HelloTemplate { name: "world" }));
    let mut buf = Vec::new();
    let _ = rsp.body.unwrap().write_body(&mut buf);
    assert_eq!(buf, b"Hello, world!");
}
