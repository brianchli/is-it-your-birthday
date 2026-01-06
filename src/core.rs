pub mod config;
pub mod handler;

use std::path::PathBuf;

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};

use chrono::Utc;
use chrono_tz::Australia::Sydney;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::core::config::{Birthday, Config};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) root: PathBuf,
    pub(crate) config: Config,
}

pub fn redirect_to(s: &str) -> Response {
    axum::http::Response::builder()
        .status(axum::http::StatusCode::PERMANENT_REDIRECT)
        .header(
            axum::http::header::LOCATION,
            format!("/is-it-{}-birthday", s),
        )
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response()
}

pub async fn serve_directory(
    mut req: Request,
    root: PathBuf,
    mut resource: PathBuf,
    birthday: &Birthday,
) -> Response {
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
