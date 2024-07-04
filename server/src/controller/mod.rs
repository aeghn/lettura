use std::sync::Arc;

use crate::{
    config::Config,
    mapper::{db::Mapper, scraper::PageScraper},
    model::ProxyConfig,
};

pub mod app_state;
pub mod article;
pub mod custom;
pub mod feed;
pub mod search;
pub mod staticfiles;

use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{
            ACCEPT, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
            ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE,
        },
        HeaderValue,
    },
    Router,
};

use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::{self, TraceLayer},
};
use tracing::{error, info, Level};

macro_rules! json_or {
    ($result:expr) => {
        match $result {
            Ok(r) => {
                let j = Json(r);
                tracing::debug!("return result: {:?}", j);
                (axum::http::StatusCode::OK, j.into_response())
            }
            Err(err) => {
                let err_str = err.to_string();
                tracing::error!("{}", err_str);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    err_str.into_response(),
                )
            }
        }
    };
}

pub(crate) use json_or;

#[derive(Clone)]
pub struct WebAppState {
    pub config: Config,
    pub page_scraper: PageScraper,
    pub pool: Arc<dyn Mapper>,
}

pub async fn serve(config: Config) {
    let pool = config.db_config.clone().into().unwrap();
    let ps = PageScraper::new(
        match config.proxy.clone() {
            Some(p) => Some(ProxyConfig {
                ip: p.ip.clone(),
                port: p.port.clone(),
                username: None,
                password: None,
            }),
            None => None,
        }
        .as_ref(),
        &pool,
    );
    if let Err(err) = ps.init().await {
        error!("unable to init: {}", err);
    }

    let state = WebAppState {
        page_scraper: ps,
        pool,
        config,
    };

    state.sync_feeds_at_interval();

    let cors_layer = CorsLayer::new()
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_methods(Any)
        .allow_origin(Any);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::DEBUG))
        .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG))
        .on_request(|req: &axum::http::Request<axum::body::Body>, _: &_| {
            // info!("request: {:?}", req);
        });

    let app = Router::new()
        .merge(article::routes())
        .merge(feed::routes())
        .merge(custom::routes())
        .merge(search::routes())
        .merge(staticfiles::routes())
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::<_>::overriding(
            ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        ))
        .layer(SetResponseHeaderLayer::<_>::overriding(
            ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("*"),
        ))
        .layer(SetResponseHeaderLayer::<_>::overriding(
            ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("*"),
        ))
        .layer(DefaultBodyLimit::disable())
        // https://stackoverflow.com/questions/73498537/axum-router-rejecting-cors-options-preflight-with-405-even-with-corslayer/
        .layer(cors_layer)
        .layer(trace_layer);

    let server_url = format!("{}:{}", "0.0.0.0", 1105);

    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    info!("server: {}", server_url);

    axum::serve(listener, app).await.unwrap();
}
