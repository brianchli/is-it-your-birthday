use std::net::{IpAddr, SocketAddr};

use axum::{extract::Path, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("BACKEND_PORT").expect("Backend port to listen on is not provided");
    let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port.parse()?);
    let listener = TcpListener::bind(&addr).await?;
    let router = axum::Router::new().route("/{*name}", get(birthday));
    Ok(axum::serve(listener, router).await?)
}

async fn birthday(Path(name): Path<String>) -> impl IntoResponse {
    name
}
