use futures_executor::block_on;
use request::request_service_client::RequestServiceClient;
use std::sync::{Arc, LazyLock, Mutex};
use tonic::transport::Channel;

pub mod request {
    tonic::include_proto!("_");
}

static CLIENT: LazyLock<Arc<Mutex<RequestServiceClient<Channel>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(init())));

const ROOT_IP: &str = "http://[::1]:50051";

fn init() -> RequestServiceClient<Channel> {
    let client = block_on(RequestServiceClient::connect(ROOT_IP)).unwrap();
    client
}

pub async fn get(
    path: String,
    host: String,
) -> Result<request::Response, Box<dyn std::error::Error>> {
    // Scope is intentionally different for mutex cloning
    let mut client = { CLIENT.lock()?.clone() };

    let request = tonic::Request::new(request::Request { path, host });
    let response = client.get(request).await?;

    Ok(response.into_inner())
}
