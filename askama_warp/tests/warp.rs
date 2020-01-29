use askama::Template;
use warp::Filter;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[tokio::test]
async fn test_warp() {
    let filter = warp::get().map(|| HelloTemplate { name: "world" });

    let res = warp::test::request().reply(&filter).await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "Hello, world!");
}
