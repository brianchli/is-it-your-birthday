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
    path::{self, PathBuf},
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
    println!("listening on http://{}", &addr);
    Ok(axum::serve(listener, router).await?)
}

async fn birthday_handler(
    State(AppState { root, config }): State<AppState>,
    path: Path<String>,
    mut req: Request,
) -> impl IntoResponse {
    match config.resolve_name(&path) {
        Some((name, birthday)) => {
            let mut resource = if let Some(p) = config.resolve_directory(name) {
                path::Path::new("/").join(p.as_path())
            } else {
                path::Path::new("/").join("default")
            };
            let today = Utc::now().with_timezone(&Sydney).date_naive();
            if birthday.matches(&today) {
                resource.push("yes/");
            } else {
                resource.push("no/");
            };
            if let Some(r) = {
                match resource.to_str() {
                    Some(resource) => resource.parse().ok(),
                    None => {
                        eprintln!("failed to convert {resource:?} to a valid UTF-8 str slice");
                        None
                    }
                }
            } {
                *req.uri_mut() = r;
            };
            match ServeDir::new(root).oneshot(req).await {
                Ok(res) => res.into_response(),
                Err(e) => {
                    eprintln!("Infallible operation failed with: {e}");
                    axum::http::StatusCode::NOT_FOUND.into_response()
                }
            }
        }
        None => {
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
