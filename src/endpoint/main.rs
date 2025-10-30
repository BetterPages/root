pub mod cache;
pub mod grpc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use grpc::get;

const BRANDING: &str = "Pages";

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    // We start a loop to continuously accept incoming connections
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let io = TokioIo::new(stream);

                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(handle_req))
                        .await
                    {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
            Err(e) => println!("Couldn't get client {e:?}"),
        };
    }
}

async fn handle_req(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri();
    if let Some(host) = req.headers().get("host") {
        let host = host.to_str().unwrap_or_default();
        // We can assume theres at least something before a : or even if there is no : this should
        // be safe.
        let host = host.split(':').next().unwrap().trim();

        let mut res = match req.method() {
            &Method::GET => {
                let (status, bytes) = get_page(path.to_string(), host.to_string()).await;
                let mut res = Response::new(Full::new(bytes));
                *res.status_mut() = StatusCode::from_u16(status.try_into().unwrap()).unwrap();

                res
            }
            _ => {
                let mut res = Response::new(Full::new(Bytes::from(
                    "Method not allowed. Please use GET requests.",
                )));
                *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                res
            }
        };

        res.headers_mut()
            .insert("Server", HeaderValue::from_static(BRANDING));

        Ok(res)
    } else {
        let mut res = Response::new(Full::new(Bytes::from("Bad request (No host header)")));
        *res.status_mut() = StatusCode::BAD_REQUEST;
        Ok(res)
    }
}

async fn get_page(path: String, host: String) -> (i32, Bytes) {
    let res = get(path, host).await.unwrap();

    return (res.status, Bytes::from(res.content));
}
