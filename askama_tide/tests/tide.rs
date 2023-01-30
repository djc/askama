use askama::Template;
use async_std::prelude::*;
use tide::{http::mime::HTML, Body, Response};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[async_std::test]
async fn template_to_response() {
    let mut res: Response = HelloTemplate { name: "world" }.into();
    assert_eq!(res.status(), 200);
    assert_eq!(res.content_type(), Some(HTML));

    let res: &mut tide::http::Response = res.as_mut();
    assert_eq!(res.body_string().await.unwrap(), "Hello, world!");
}

#[async_std::test]
async fn template_to_body() {
    let mut body: Body = HelloTemplate { name: "world" }.try_into().unwrap();
    assert_eq!(body.mime(), &HTML);
    let mut body_string = String::new();
    body.read_to_string(&mut body_string).await.unwrap();
    assert_eq!(body_string, "Hello, world!");
}
