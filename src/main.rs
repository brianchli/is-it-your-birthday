mod config;

use axum::{
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::get,
};
use chrono::Utc;
use chrono_tz::Australia::Sydney;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use tokio::net::TcpListener;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::config::Config;

#[derive(Clone)]
struct AppState {
    root: PathBuf,
    config: Config,
}

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
    Ok(axum::serve(listener, router).await?)
}

async fn birthday_handler(
    State(AppState { root, config }): State<AppState>,
    path: Path<String>,
    mut req: Request,
) -> impl IntoResponse {
    match config.resolve_name(&path) {
        Some((name, birthday)) => {
            let mut resource = if let Some(paths) = &config.path
                && let Some(p) = paths.get(&String::from(name))
            {
                p.to_path_buf()
            } else {
                let mut p = root.clone();
                p.push("default");
                p
            };
            let today = Utc::now().with_timezone(&Sydney).date_naive();
            if birthday.matches(&today) {
                resource.push("yes");
            } else {
                resource.push("no");
            };
            *req.uri_mut() = "/".parse().unwrap();
            ServeDir::new(resource).oneshot(req).await.unwrap()
        } // this will return a 404 without uri parsing
        None => ServeDir::new(root).oneshot(req).await.unwrap(),
    }
}
