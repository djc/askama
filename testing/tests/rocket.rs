#![cfg(feature = "with-rocket")]
#![feature(proc_macro_hygiene, decl_macro)]

extern crate askama;
#[macro_use]
extern crate rocket;

use askama::Template;

use rocket::http::{ContentType, Status};
use rocket::local::Client;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[get("/")]
fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

#[test]
fn test_rocket() {
    let rocket = rocket::ignite().mount("/", routes![hello]);
    let client = Client::new(rocket).unwrap();
    let mut rsp = client.get("/").dispatch();
    assert_eq!(rsp.status(), Status::Ok);
    assert_eq!(rsp.content_type(), Some(ContentType::HTML));
    assert_eq!(rsp.body_string().unwrap(), "Hello, world!");
}
