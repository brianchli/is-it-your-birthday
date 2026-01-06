mod core;

use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::get,
};
use std::{
    net::{IpAddr, SocketAddr},
    path::{self, PathBuf},
};
use tokio::net::TcpListener;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::core::{
    AppState,
    config::{Actions, Config},
    handler::Handler,
    redirect_to, serve_directory,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("BACKEND_PORT").expect("Backend port to listen on is not provided");
    let root = PathBuf::from(std::env::var("ASSET_PATH").expect("Asset location is not provided"));
    let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port.parse()?);
    let listener = TcpListener::bind(&addr).await?;
    let config = Config::new().await?;
    let router = axum::Router::new()
        .route("/{*name}", get(birthday_handler))
        .with_state(AppState { root, config });
    println!("listening on http://{}", &addr);
    Ok(axum::serve(listener, router).await?)
}

async fn birthday_handler(
    State(AppState { root, config }): State<AppState>,
    path: Path<String>,
    mut req: Request,
) -> impl IntoResponse {
    match Handler::execute(&config, &path) {
        Some((Actions::Resolve(..), Some(p), Some(birthday))) => {
            let resource = path::Path::new("/").join(p.as_path());
            serve_directory(req, root, resource, birthday).await
        }
        Some((Actions::Resolve(..), None, Some(birthday))) => {
            let resource = path::Path::new("/").join("default");
            serve_directory(req, root, resource, birthday).await
        }
        Some((Actions::Redirect { to }, ..)) => redirect_to(&to),
        _ => {
            *req.uri_mut() = "/empty/".parse().unwrap();
            match ServeDir::new(root).oneshot(req).await {
                Ok(res) => res.into_response(),
                Err(e) => {
                    eprintln!("Infallible operation failed with: {e}");
                    axum::http::StatusCode::NOT_FOUND.into_response()
                }
            }
        }
    }
}
