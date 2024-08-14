use askama_rocket::Template;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[rocket::get("/")]
fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

#[tokio::test]
async fn test_rocket() {
    let rocket = rocket::build()
        .mount("/", rocket::routes![hello])
        .ignite()
        .await
        .unwrap();
    let client = Client::untracked(rocket).await.unwrap();
    let rsp = client.get("/").dispatch().await;
    assert_eq!(rsp.status(), Status::Ok);
    assert_eq!(rsp.content_type(), Some(ContentType::HTML));
    assert_eq!(rsp.into_string().await.as_deref(), Some("Hello, world!"));
}
