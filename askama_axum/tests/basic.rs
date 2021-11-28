use askama::Template;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use tower::util::ServiceExt;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

#[tokio::test]
async fn template_to_response() {
    let app = Router::new().route("/", get(hello));

    let res = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let headers = res.headers();
    assert_eq!(headers["Content-Type"], "text/html; charset=utf-8");

    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, world!");
}
