use serde::{Deserialize, Serialize};

use super::db::{Article, Feed, FeedLog};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkReadReq {
    pub is_read: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkStartReq {
    pub is_starred: bool,
}

#[derive(Debug, Deserialize)]
pub struct ForwardUrlReq {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleFilterReq {
    pub feed_id: Option<String>,
    pub is_today: Option<bool>,
    pub is_starred: Option<bool>,
    pub is_read: Option<bool>,
    pub keyword: Option<String>,
    pub cursor: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSortReq {
    item_type: String,
    pub uuid: String,
    pub folder_uuid: String,
    pub sort: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedsCountReq {
    pub filter: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedsUpdateSyncIntervalReq {
    pub interval: u32,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct FeedSortRsp {
    parent_id: String,
    child_uuid: String,
    sort: i32,
}

#[derive(Deserialize)]
pub struct FeedAddReq {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FeedAddRsp {
    pub feed: Feed,
    pub article_count: u64,
}

#[derive(Deserialize)]
pub struct FeedFetchReq {
    pub url: String,
}

#[derive(Deserialize)]
pub struct FolderAddReq {
    pub name: String,
}

#[derive(Deserialize)]
pub struct FeedMoveReq {
    pub channel_uuid: String,
    pub folder_uuid: String,
    pub sort: i32,
}

#[derive(Deserialize)]
pub struct FolderRenameReq {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionMeta {
    pub total: i64,
    pub today: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchReq {
    pub query: String,
    pub limit: Option<i32>,
    pub cursor: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct SearchArticleRsp {
    #[serde(flatten)]
    pub article: Article,
}

#[derive(Debug, Serialize)]
pub struct FeedResRsp {
    #[serde(flatten)]
    pub feed: Feed,
    #[serde(flatten)]
    pub feed_log: FeedLog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkAllReadReq {
    pub uuid: Option<String>,
    pub is_today: Option<bool>,
    pub is_all: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedSyncResult {
    pub feed_id: String,
    pub title: String,
    pub count: u64,
    pub err_info: String,
}
