use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    controller::{json_or, WebAppState},
    mapper::feed_rs::map_feed_to_feed,
    model::{
        db::FeedTypeMark,
        dto::{
            FeedAddReq, FeedFetchReq, FeedMoveReq, FeedSortReq, FeedsCountReq,
            FeedsUpdateSyncIntervalReq, FolderAddReq, FolderRenameReq, MarkAllReadReq,
        },
    },
};

pub async fn handle_sync_feed(
    app_state: State<WebAppState>,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    let res = app_state
        .sync_feeds(Some(&uuid.to_string().as_str()))
        .await
        .map(|e| {
            let mut hm = HashMap::new();
            e.into_iter().for_each(|f| {
                hm.insert(f.feed_id, (f.title, f.count, f.err_info));
            });
            hm
        });

    json_or!(res)
}

pub async fn handle_get_subscribes(app_state: State<WebAppState>) -> impl IntoResponse {
    let result: Vec<serde_json::Value> = app_state
        .pool
        .get_all_feeds()
        .await
        .unwrap()
        .iter()
        .map(|e| {
            let is_folder = e.item_type == FeedTypeMark::Folder;
            json!({
            "uuid": e.id.clone(),
            "title": e.title.clone(),
            "link": e.link.clone(),
            "feed_url": e.feed_url.clone(),
            "logo": e.logo.clone(),
            "description": e.description.clone(),
            "pub_date": e.create_time,
            "health_status": 1,
            "failure_reason": "",
            "sort": e.sort.clone(),
            "sync_interval_sec": e.sync_interval_sec,
            "last_sync_date": "1999-12-31",
            "create_time": e.create_time,
            "update_time": e.update_time,
            "parent_id": e.parent_id,
            "folder_uuid": if is_folder {Some(e.id.clone())} else {None},
            "folder_name": if is_folder {Some(e.title.clone())} else {None} ,
            "item_type": e.item_type,
            "is_expanded": false
            })
        })
        .collect();

    json_or!(Ok::<Vec<serde_json::Value>, anyhow::Error>(result))
}

pub async fn handle_delete_feed(
    app_state: State<WebAppState>,
    uid: Path<String>,
) -> impl IntoResponse {
    let results = app_state.pool.delete_feed(uid.0.as_str()).await;

    json_or!(results)
}

pub async fn handle_get_feed_log(
    app_state: State<WebAppState>,
    feed_id: Path<String>,
) -> impl IntoResponse {
    let results = app_state.pool.get_feed_logs(&feed_id).await;

    json_or!(results)
}

pub async fn handle_update_feed_sort(
    app_state: State<WebAppState>,
    body: Json<Vec<FeedSortReq>>,
) -> impl IntoResponse {
    json_or!(app_state.pool.update_feed_sort(&body).await)
}

pub async fn move_channel_into_folder(
    app_state: State<WebAppState>,
    Json(req): Json<FeedMoveReq>,
) -> impl IntoResponse {
    let r = app_state
        .pool
        .move_channel_into_folder(&req.channel_uuid.as_str(), &req.folder_uuid.as_str())
        .await;
    json_or!(r)
}

pub async fn add_feed(
    app_state: State<WebAppState>,
    Json(req): Json<FeedAddReq>,
) -> impl IntoResponse {
    let res = app_state.add_feed(req).await;
    json_or!(res)
}

pub async fn create_folder(
    app_state: State<WebAppState>,
    Json(req): Json<FolderAddReq>,
) -> impl IntoResponse {
    let res = app_state.add_folder(&req.name).await.map(|m| vec![m]);
    json_or!(res)
}

pub async fn delete_folder(app_state: State<WebAppState>, req: Path<String>) -> impl IntoResponse {
    let results = app_state.pool.delete_feed(&req).await;

    json_or!(results)
}

pub async fn sync_folder(
    app_state: State<WebAppState>,
    uuid_name: Path<(String, String)>,
) -> impl IntoResponse {
    let un = uuid_name.0;
    let res = app_state.sync_feeds(Some(&un.0)).await.map(|e| {
        let mut hm = HashMap::new();
        e.into_iter().for_each(|f| {
            hm.insert(f.feed_id, (f.title, f.count, f.err_info));
        });
        hm
    });

    json_or!(res)
}

pub async fn update_folder_name(
    app_state: State<WebAppState>,
    req: Json<FolderRenameReq>,
) -> impl IntoResponse {
    let res = app_state
        .pool
        .update_folder_name(&req.uuid, &req.name)
        .await;

    json_or!(res)
}

pub async fn get_folders(app_state: State<WebAppState>) -> impl IntoResponse {
    json_or!(Ok::<(), anyhow::Error>(()))
}

pub async fn update_feed_sync_interval(
    app_state: State<WebAppState>,
    query: Query<FeedsUpdateSyncIntervalReq>,
) -> impl IntoResponse {
    let res = app_state
        .pool
        .update_feed_sync_interval(&query.id, query.interval)
        .await;

    json_or!(res)
}

pub async fn update_icon(
    app_state: State<WebAppState>,
    uu: Json<(String, String)>,
) -> impl IntoResponse {
    json_or!(Ok::<(), anyhow::Error>(()))
}

pub async fn handle_get_unread_total_per_feed(
    app_state: State<WebAppState>,
    query: Query<FeedsCountReq>,
) -> impl IntoResponse {
    let count_map = app_state.pool.count_per_feed(&query.0).await;
    json_or!(count_map)
}

pub async fn fetch_feed(
    app_state: State<WebAppState>,
    Json(req): Json<FeedFetchReq>,
) -> impl IntoResponse {
    let url = req.url;

    let channel_uuid = Uuid::new_v4().hyphenated().to_string();
    let res = app_state.page_scraper.parse_feed(&url).await;

    json_or!(res.map(|feed| { map_feed_to_feed(&channel_uuid, None, &0, url.as_str(), &feed,) }))
}

pub async fn get_collection_metas(app_state: State<WebAppState>) -> impl IntoResponse {
    let res = app_state.pool.get_collection_metas().await;
    json_or!(res)
}

pub async fn handle_mark_as_read(
    app_state: State<WebAppState>,
    Json(body): Json<MarkAllReadReq>,
) -> impl IntoResponse {
    json_or!(app_state.mark_as_read(&body).await)
}

pub fn routes() -> Router<WebAppState> {
    Router::new()
        .route("/api/feeds/:uuid/sync", get(handle_sync_feed))
        .route("/api/feeds", get(handle_get_subscribes))
        .route("/api/subscribes", get(handle_get_subscribes))
        .route("/api/feeds/:uuid", delete(handle_delete_feed))
        .route("/api/update-feed-sort", post(handle_update_feed_sort))
        .route(
            "/api/move-channel-into-folder",
            post(move_channel_into_folder),
        )
        .route("/api/add-feed", post(add_feed))
        .route("/api/create-folder", post(create_folder))
        .route("/api/delete-folder/:uuid", post(delete_folder))
        .route("/api/update-folder", post(update_folder_name))
        .route("/api/get-folders", get(get_folders))
        .route("/api/update-icon/:uuid/:url", post(update_icon))
        .route("/api/unread-total", get(handle_get_unread_total_per_feed))
        .route("/api/fetch-feed", post(fetch_feed))
        .route("/api/collection-metas", get(get_collection_metas))
        .route("/api/feed-log/:uuid", get(handle_get_feed_log))
        .route("/api/mark-all-as-read", post(handle_mark_as_read))
        .route(
            "/api/update-feed-sync-interval",
            post(update_feed_sync_interval),
        )
}
