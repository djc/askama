use hyper::{
    body::{self, Body, HttpBody},
    http, Server,
};
use routerify::{Router, RouterService};
use std::net::SocketAddr;
use tokio::sync::oneshot::{self, Sender};

pub struct Serve {
    addr: SocketAddr,
    tx: Sender<()>,
}

impl Serve {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn new_request(&self, method: &str, route: &str) -> http::request::Builder {
        http::request::Request::builder()
            .method(method.to_ascii_uppercase().as_str())
            .uri(format!("http://{}{}", self.addr(), route))
    }

    pub fn shutdown(self) {
        self.tx.send(()).unwrap();
    }
}

pub async fn serve<B, E>(router: Router<B, E>) -> Serve
where
    B: HttpBody + Send + Sync + 'static,
    E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    <B as HttpBody>::Data: Send + Sync + 'static,
    <B as HttpBody>::Error: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
{
    let service = RouterService::new(router).unwrap();
    let server = Server::bind(&([127, 0, 0, 1], 0).into()).serve(service);
    let addr = server.local_addr();

    let (tx, rx) = oneshot::channel::<()>();

    let graceful_server = server.with_graceful_shutdown(async {
        rx.await.unwrap();
    });

    tokio::spawn(async move {
        graceful_server.await.unwrap();
    });

    Serve { addr, tx }
}

pub async fn into_text(body: Body) -> String {
    String::from_utf8_lossy(&body::to_bytes(body).await.unwrap()).to_string()
}
