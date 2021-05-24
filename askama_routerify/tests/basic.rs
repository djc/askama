use self::support::{into_text, serve};
use askama::Template;
use hyper::{Body, Client, Request, Response};
use routerify::Router;
use std::convert::Infallible;

mod support;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(HelloTemplate { name: "world" }.into())
}

#[tokio::test]
async fn test_routerify() {
    let router: Router<Body, Infallible> = Router::builder().get("/", hello).build().unwrap();

    let serve = serve(router).await;

    let resp = Client::new()
        .request(serve.new_request("GET", "/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(into_text(resp.into_body()).await, "Hello, world!");

    serve.shutdown();
}
