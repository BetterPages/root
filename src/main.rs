pub mod config;
mod resolver;

use request::{
    Request, Response,
    request_service_server::{RequestService, RequestServiceServer},
};
use std::fs;
use tonic::{Status, transport::Server};

use config::GLOBAL_404;
use resolver::{resolve_domain, resolve_path};

pub mod request {
    tonic::include_proto!("_");
}

#[derive(Debug, Default)]
pub struct RequestGreeter {}

#[tonic::async_trait]
impl RequestService for RequestGreeter {
    async fn get(
        &self,
        request: tonic::Request<Request>,
    ) -> Result<tonic::Response<Response>, Status> {
        let req = request.into_inner();

        let domain = resolve_domain(&req.host);
        let fs_path = resolve_path(&domain, &req.path);
        let data = match fs_path {
            Some(ref fs_path) => {
                if fs_path.starts_with(&domain) {
                    fs::read(fs_path).unwrap()
                } else {
                    // Use the global 404 to mask the attacks surface (Make the attackers think
                    // they are successful, taking up their time instead of letting them target
                    // other potential attack surfaces).
                    GLOBAL_404.to_vec()
                }
            }
            None => GLOBAL_404.to_vec(),
        };

        let reply = Response {
            status: 200,
            content: data,
        };

        Ok(tonic::Response::new(reply))
    }
}

#[tokio::main]
async fn main() {
    let addr = "[::1]:50051".parse().unwrap();
    let requestserver = RequestGreeter::default();

    println!("Running on port 50051.");

    Server::builder()
        .add_service(RequestServiceServer::new(requestserver))
        .serve(addr)
        .await
        .unwrap();
}
