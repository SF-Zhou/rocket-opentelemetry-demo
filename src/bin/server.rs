use std::{net::SocketAddr, str::FromStr};

use opentelemetry_sdk::Resource;
use rocket_opentelemetry_demo::*;
use tracing::{instrument, Instrument};

#[instrument]
async fn func1() -> String {
    tracing::info!("enter func1");
    tokio::time::sleep(std::time::Duration::from_millis(233)).await;
    "hello".into()
}

#[instrument]
async fn func2() -> String {
    tracing::info!("enter func2");
    tokio::time::sleep(std::time::Duration::from_millis(666)).await;
    "world".into()
}

#[instrument]
async fn main_func() -> String {
    let a = func1().await;
    let b = func2().await;
    format!("{} {}!", a, b)
}

#[rocket::get("/")]
async fn index(span: TracingSpan) -> String {
    main_func().instrument(span.0).await
}

#[rocket::launch]
async fn rocket() -> _ {
    let endpoint = SocketAddr::from_str("127.0.0.1:4317").unwrap();
    let resource = Resource::builder().with_service_name("demo").build();
    init_opentelemetry(&endpoint, resource);

    rocket::build()
        .mount("/", rocket::routes![index])
        .attach(TracingFairing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::uri;

    #[rocket::async_test]
    async fn test_server() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(uri!(index)).dispatch().await;
        assert_eq!(response.status(), rocket::http::Status::Ok);
    }
}
