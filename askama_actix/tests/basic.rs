use actix_web::http::header::{CONTENT_TYPE};
use actix_web::web;
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[actix_rt::test]
async fn test_actix_web() {
    let srv = actix_test::start(|| {
        actix_web::App::new()
            .service(web::resource("/").to(|| async { HelloTemplate { name: "world" } }))
    });

    let request = srv.get("/");
    let mut response = request.send().await.unwrap();
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    let  body = response.body().await.unwrap();
    assert_eq!(body, "Hello, world!");
}

#[actix_rt::test]
async fn test_actix_web_responder() {
    let srv = actix_test::start(|| {
        actix_web::App::new().service(web::resource("/").to(|| async {
            let name = "world".to_owned();
            HelloTemplate { name: &name }.to_response()
        }))
    });

    let request = srv.get("/");
    let mut response = request.send().await.unwrap();
    assert!(response.status().is_success());
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    let body = response.body().await.unwrap();
    assert_eq!(body, "Hello, world!");
}
