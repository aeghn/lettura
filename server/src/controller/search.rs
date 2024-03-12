use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::{
    model::dto::{SearchArtileRsp, SearchReq},
    tool::htmlconverter::replace_image_article,
};

use super::{json_or, WebAppState};

pub async fn handle_search(
    app_state: State<WebAppState>,
    search: Query<SearchReq>,
) -> impl IntoResponse {
    let result: Result<Vec<SearchArtileRsp>, anyhow::Error> =
        app_state.pool.global_search(search.0).await.map(|e| {
            e.into_iter()
                .map(|a| SearchArtileRsp {
                    article: replace_image_article(a.article),
                })
                .collect()
        });

    json_or!(result)
}

pub fn routes() -> Router<WebAppState> {
    Router::new().route("/api/search", get(handle_search))
}
