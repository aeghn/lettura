pub mod postgres;

use std::collections::HashMap;

use async_trait::async_trait;

use crate::model::{
    alias::RResult,
    db::{Article, Feed, FeedLog, WebCache},
    dto::{
        ArticleFilterReq, CollectionMeta, FeedSortReq, FeedsCountReq, MarkAllReadReq,
        SearchArtileRsp, SearchReq,
    },
};

#[async_trait]
pub trait Mapper: ArticleMapper + FeedMapper + Send + Sync {
    async fn global_search(&self, search: SearchReq) -> RResult<Vec<SearchArtileRsp>>;

    async fn get_web_cache(&self, url: &str) -> RResult<WebCache>;
    async fn set_web_cache(&self, url: &WebCache) -> RResult<u64>;
}

#[async_trait]
pub trait ArticleMapper {
    async fn get_article_with_uuid(&self, uuid: &str) -> RResult<Article>;
    async fn get_article_with_url(&self, url: &str) -> RResult<Article>;

    async fn set_article_content_with_url(&self, url: &str, content: &str) -> RResult<()>;

    async fn update_article_read_status(&self, uuid: &str, is_read: bool) -> RResult<usize>;

    async fn update_article_star_status(&self, uuid: &str, is_starred: bool) -> RResult<usize>;
    async fn get_articles(&self, filter: &ArticleFilterReq) -> RResult<Vec<Article>>;

    async fn add_articles(&self, articles: &Vec<Article>) -> RResult<u64>;
}

#[async_trait]
pub trait FeedMapper {
    async fn get_sub_ids(&self, id: &str) -> RResult<Vec<String>>;
    async fn get_sub_ids_as_sql_seg(&self, id: &str) -> RResult<String> {
        let inner = self
            .get_sub_ids(&id)
            .await?
            .into_iter()
            .map(|e| format!("'{}'", e))
            .collect::<Vec<String>>()
            .join(",");
        Ok(format!("({})", inner))
    }

    async fn mark_as_read(&self, id: &MarkAllReadReq) -> RResult<()>;

    async fn get_sub_true_feeds(&self, id: &str) -> RResult<Vec<Feed>>;

    async fn get_all_feeds(&self) -> RResult<Vec<Feed>>;

    async fn write_log(&self, log: &FeedLog) -> RResult<()>;
    async fn get_feed_logs(&self, id: &str) -> RResult<Vec<FeedLog>>;
    async fn get_last_success_feed_log(&self, id: &str) -> Option<FeedLog>;

    async fn delete_feed(&self, id: &str) -> RResult<u64>;
    async fn update_feed_sort(&self, sorts: &Vec<FeedSortReq>) -> RResult<u64>;
    async fn add_feed(&self, feed: &Feed) -> RResult<()>;

    async fn count_per_feed(&self, req: &FeedsCountReq) -> RResult<HashMap<String, i64>>;

    async fn get_collection_metas(&self) -> RResult<CollectionMeta>;

    async fn update_folder_name(&self, uuid: &str, name: &str) -> RResult<()>;
    async fn update_feed_sync_interval(&self, id: &str, interval: u32) -> RResult<u64>;

    async fn move_channel_into_folder(&self, sub_id: &str, parent_id: &str) -> RResult<()>;
}
