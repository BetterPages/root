pub mod cache;
pub mod compression;
pub mod grpc;
pub mod types;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use mimetype_detector::detect;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use cache::{get_cache_entry, insert_cache_entry};
use compression::{CompressionMethod, compress, get_header_name};
use grpc::get;
use types::*;

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
    let path = req.uri().to_string();
    if let Some(host) = req.headers().get("host") {
        let host = host.to_str().unwrap_or_default();
        // We can assume theres at least something before a : or even if there is no : this should
        // be safe.
        let host = host.split(':').next().unwrap().trim();

        let mut res = match req.method() {
            &Method::GET => {
                let compression = if let Some(header) = req.headers().get("Accept-Encoding")
                    && let Ok(header) = header.to_str()
                {
                    if header.contains("br") {
                        CompressionMethod::BROTLI
                    } else if header.contains("gzip") {
                        CompressionMethod::GZIP
                    } else {
                        CompressionMethod::NONE
                    }
                } else {
                    CompressionMethod::NONE
                };

                let page = get_page(path.clone(), host.to_string(), compression).await;

                let mut res = Response::new(Full::new(Bytes::from(page.content)));
                if compression != CompressionMethod::NONE {
                    res.headers_mut().append(
                        "Content-Encoding",
                        HeaderValue::from_static(get_header_name(compression)),
                    );
                };

                *res.status_mut() = StatusCode::from_u16(page.status).unwrap();
                res.headers_mut()
                    .insert("Content-Type", HeaderValue::from_str(&page.mime).unwrap());
                res.headers_mut()
                    .insert("Vary", HeaderValue::from_static("Content-Encoding"));

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
        res.headers_mut()
            .insert("x-frame-options", HeaderValue::from_static("deny"));
        res.headers_mut()
            .insert("Vary", HeaderValue::from_static("Accept-Encoding"));

        Ok(res)
    } else {
        let mut res = Response::new(Full::new(Bytes::from("Bad request (No host header)")));
        *res.status_mut() = StatusCode::BAD_REQUEST;
        Ok(res)
    }
}

async fn get_page(mut path: String, host: String, compression: CompressionMethod) -> Page {
    // Sanitize URLs
    if let Some(splitted_path) = path.split_once("#") {
        path = splitted_path.0.to_string();
    };
    if let Some(splitted_path) = path.split_once("?") {
        path = splitted_path.0.to_string();
    };

    match get_cache_entry(path.clone(), host.clone(), compression) {
        Some(cache_entry) => cache_entry,
        None => {
            let res = get(path.clone(), host.clone()).await.unwrap();
            let mime = if path.ends_with(".css") {
                "text/css".into()
            } else if path.ends_with(".js") {
                "text/javascript".into()
            } else if res.content.starts_with(b"<") {
                "text/html".into()
            } else {
                detect(&res.content).mime().to_string()
            };

            let page = Page {
                status: res.status.try_into().unwrap(),
                content: compress(compression, &res.content),
                mime: mime,
            };

            insert_cache_entry(path, host, compression, page.clone());
            page
        }
    }
}
