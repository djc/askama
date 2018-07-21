//#![cfg(feature = "actix-web")]

#[macro_use]
extern crate askama;
extern crate actix_web;
extern crate bytes;

use bytes::Bytes;
use askama::Template;
use actix_web::test;
use actix_web::HttpMessage;
use actix_web::http::header::CONTENT_TYPE;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_actix_web() {
    let mut srv = test::TestServer::new(|app| {
        app.handler(|_| HelloTemplate {
            name: "world"
        })
    });

    let request = srv.get().finish().unwrap();
    let response = srv.execute(request.send()).unwrap();
    assert!(response.status().is_success());
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), "text/html");

    let bytes = srv.execute(response.body()).unwrap();
    assert_eq!(bytes, Bytes::from_static("Hello, world!".as_ref()));
}
