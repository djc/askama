use askama::Template;
use gotham::state::State;
use gotham::test::TestServer;
use hyper::StatusCode;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

fn hello(state: State) -> (State, HelloTemplate<'static>) {
    (state, HelloTemplate { name: "world" })
}

#[test]
fn test_gotham() {
    let test_server = TestServer::new(|| Ok(hello)).expect("Failed to mount test router");

    let res = test_server
        .client()
        .get("http://localhost/")
        .perform()
        .expect("Failed to send request to gotham");

    assert_eq!(res.status(), StatusCode::OK);
    {
        let headers = res.headers();
        let content_type = headers
            .get("content-type")
            .expect("Response did not contain content-type header");
        assert_eq!(
            content_type.to_str().unwrap(),
            mime::TEXT_HTML_UTF_8.to_string()
        );
    }

    let body = res.read_utf8_body().expect("failed to read response body");
    assert_eq!(&body, "Hello, world!");
}
