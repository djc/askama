use askama::Template;
use iron::{status, Response};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "hello.txt")]
struct HelloTextTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_iron() {
    let rsp = Response::with((status::Ok, HelloTemplate { name: "world" }));
    let mut buf = Vec::new();
    let _ = rsp.body.unwrap().write_body(&mut buf);
    assert_eq!(buf, b"Hello, world!");

    let content_type = rsp.headers.get::<iron::headers::ContentType>().unwrap();
    assert_eq!(format!("{}", content_type), "text/html; charset=utf-8");
}

#[test]
fn test_iron_non_html() {
    let rsp = Response::with((status::Ok, HelloTextTemplate { name: "world" }));
    let mut buf = Vec::new();
    let _ = rsp.body.unwrap().write_body(&mut buf);
    assert_eq!(buf, b"Hello, world!");

    let content_type = rsp.headers.get::<iron::headers::ContentType>();
    assert_eq!(content_type, None);
}
