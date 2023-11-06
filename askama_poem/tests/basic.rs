use askama::Template;
use poem::handler;
use poem::{get, http::StatusCode, test::TestClient, Route};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[handler]
async fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

#[tokio::test]
async fn template_to_response() {
    let app = Route::new().at("/", get(hello));
    let cli = TestClient::new(app);

    let res = cli.get("/").send().await;
    assert_eq!(res.0.status(), StatusCode::OK);

    let headers = res.0.headers();
    assert_eq!(headers["Content-Type"], "text/html; charset=utf-8");

    let body = res.0.into_body().into_bytes().await.unwrap();
    assert_eq!(&body[..], b"Hello, world!");
}
