use std::collections::HashMap;

use axum::{
    body::{self, Body},
    extract::{Path, Query, State},
    http::HeaderName,
    response::{ErrorResponse, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use reqwest::header;

use crate::{
    controller::{json_or, WebAppState},
    model::dto::{ArticleFilterReq, ForwardUrlReq, MarkReadReq, MarkStartReq},
    tool::htmlconverter::replace_image_article,
};

pub async fn handle_get_article_detail(
    app_state: State<WebAppState>,
    uuid: Path<String>,
) -> impl IntoResponse {
    let res = app_state
        .pool
        .get_article_with_uuid(uuid.as_str())
        .await
        .map(|e| replace_image_article(e));

    json_or!(res)
}

pub async fn handle_update_article_read_status(
    app_state: State<WebAppState>,
    uuid: Path<String>,
    body: Json<MarkReadReq>,
) -> impl IntoResponse {
    let res = app_state
        .pool
        .update_article_read_status(uuid.as_str(), body.is_read)
        .await;

    json_or!(res)
}

pub async fn handle_update_article_star_status(
    app_state: State<WebAppState>,
    uuid: Path<String>,
    body: Json<MarkStartReq>,
) -> impl IntoResponse {
    let res = app_state
        .pool
        .update_article_star_status(uuid.as_str(), body.is_starred)
        .await;

    json_or!(res)
}

pub async fn handle_get_article_best_image(
    app_state: State<WebAppState>,
    query: Query<ForwardUrlReq>,
) -> impl IntoResponse {
    let res = app_state
        .page_scraper
        .get_first_image_or_og_image(&(query.url.to_string()))
        .await;

    json_or!(res)
}

pub async fn handle_get_article_source(
    app_state: State<WebAppState>,
    query: Query<ForwardUrlReq>,
) -> impl IntoResponse {
    let r = app_state
        .fetch_article_and_cache(&query.0)
        .await
        .map(|e| crate::tool::htmlconverter::replace_image(e.as_str()));

    json_or!(r)
}

pub async fn handle_get_cache(
    app_state: State<WebAppState>,
    query: Query<ForwardUrlReq>,
) -> impl IntoResponse {
    let r = app_state.fetch_attachment_and_cache(&query.0).await;

    match r {
        Ok((cache, body)) => {
            let ct = cache.content_type.clone();
            let body: Body = body;

            let headers = [(header::CONTENT_TYPE, ct)];
            Ok::<([(HeaderName, String); 1], Body), ErrorResponse>((headers, body))
        }
        Err(err) => {
            let err_str = err.to_string();
            tracing::error!("{}", err_str);
            Ok::<([(HeaderName, String); 1], Body), ErrorResponse>((
                [(header::CONTENT_TYPE, "".to_owned())],
                Body::empty(),
            ))
        }
    }
}

pub async fn handle_get_image(
    app_state: State<WebAppState>,
    url: Path<String>,
) -> impl IntoResponse {
    let r = app_state
        .fetch_attachment_and_cache(&ForwardUrlReq { url: url.0 })
        .await;

    match r {
        Ok((cache, body)) => {
            let ct = cache.content_type.clone();
            let body: Body = body::Body::from(body);

            let headers = [(header::CONTENT_TYPE, ct)];
            (
                axum::http::StatusCode::OK,
                Ok::<([(HeaderName, String); 1], Body), ErrorResponse>((headers, body))
                    .into_response(),
            )
        }
        Err(err) => {
            let err_str = err.to_string();
            tracing::error!("==== {}", err_str);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                err_str.into_response(),
            )
        }
    }
}

pub async fn handle_articles(
    app_state: State<WebAppState>,
    query: Query<ArticleFilterReq>,
) -> impl IntoResponse {
    let filter = ArticleFilterReq {
        feed_id: query.feed_id.clone(),
        is_today: query.is_today.clone(),
        is_starred: query.is_starred.clone(),
        is_read: query.is_read.clone(),
        cursor: query.cursor.clone(),
        limit: query.limit.clone(),
    };

    let res = app_state.pool.get_articles(&filter).await.map(|v| {
        let mut hm = HashMap::new();
        hm.insert("list", v);
        hm
    });

    json_or!(res)
}

pub fn routes() -> Router<WebAppState> {
    Router::new()
        .route("/api/articles/:uuid", get(handle_get_article_detail))
        .route(
            "/api/articles/:uuid/read",
            post(handle_update_article_read_status),
        )
        .route(
            "/api/articles/:uuid/star",
            post(handle_update_article_star_status),
        )
        .route("/api/image-proxy", get(handle_get_cache))
        .route("/images/:url", get(handle_get_image))
        .route("/api/article-proxy", get(handle_get_article_source))
        .route("/api/articles", get(handle_articles))
}
