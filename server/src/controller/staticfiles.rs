use axum::{
    http::{header, Uri},
    response::IntoResponse,
    routing::{get, Router},
};
use rust_embed::RustEmbed;

use super::WebAppState;

pub fn routes() -> Router<WebAppState> {
    Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/assets/*file", get(static_handler))
        .route("/*file", get(static_handler))
        .fallback_service(get(index_handler))
}

async fn index_handler() -> impl IntoResponse {
    let content = Asset::get("index.html").unwrap();
    let mime = mime_guess::from_path("index.html").first_or_octet_stream();
    ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("dist/") {
        path = path.replace("dist/", "");
    };

    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => index_handler().await.into_response(),
    }
}

#[derive(RustEmbed)]
#[folder = "../web-dist"]
struct Asset;
