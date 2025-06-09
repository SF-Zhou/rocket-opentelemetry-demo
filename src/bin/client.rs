use std::{net::SocketAddr, str::FromStr};

use opentelemetry_sdk::Resource;
use rocket_opentelemetry_demo::*;

#[tracing::instrument]
async fn send_request() {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8000")
        .headers(injecte_into_header_map())
        .send()
        .await
        .unwrap();
    let body = response.text().await.unwrap();
    tracing::info!("rsp: {}", body);
    assert_eq!(body, "hello world!");
}

#[tokio::main]
async fn main() {
    let endpoint = SocketAddr::from_str("127.0.0.1:4317").unwrap();
    let resource = Resource::builder().with_service_name("client").build();
    init_opentelemetry(&endpoint, resource);

    send_request().await;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
}
