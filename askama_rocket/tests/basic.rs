use askama::Template;

use futures_lite::future::block_on;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rocket::Responder;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[derive(Responder)]
struct HelloResponder<'a> {
    template: HelloTemplate<'a>,
}

#[rocket::get("/")]
fn hello() -> HelloTemplate<'static> {
    HelloTemplate { name: "world" }
}

#[test]
fn test_rocket() {
    block_on(async {
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
    });
}
