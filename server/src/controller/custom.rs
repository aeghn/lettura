use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{controller::json_or, model::custom::UserConfig};

use super::WebAppState;

pub async fn handle_get_user_config(_app_state: State<WebAppState>) -> impl IntoResponse {
    let user_config = UserConfig::default();

    json_or!(Ok::<UserConfig, anyhow::Error>(user_config))
}

pub async fn handle_update_user_config(
    app_state: State<WebAppState>,
    user_cfg: Json<UserConfig>,
) -> impl IntoResponse {
    json_or!(Ok::<(), anyhow::Error>(()))
}

pub fn routes() -> Router<WebAppState> {
    Router::new()
        .route("/api/user-config", get(handle_get_user_config))
        .route("/api/user-config", post(handle_update_user_config))
}
