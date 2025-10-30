use request::{
    Request, Response,
    request_service_server::{RequestService, RequestServiceServer},
};
use tonic::{Status, transport::Server};

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

        let reply = Response {
            status: 200,
            content: format!(
                "<html><body>This works. The host is {}, and the path is {}.</body></html>",
                req.host, req.path
            )
            .into(),
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
