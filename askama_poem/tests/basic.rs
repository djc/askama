use askama::Template;
use poem::{handler, test::TestClient, Route};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[handler]
fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

#[tokio::test]
async fn test_poem() {
    let app = Route::new().at("/", hello);
    let cli = TestClient::new(app);

    let resp = cli.get("/").send().await;
    resp.assert_status_is_ok();
    resp.assert_text("Hello, world!").await;
}
