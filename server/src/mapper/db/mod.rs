pub mod postgres;

use std::collections::HashMap;

use async_trait::async_trait;
use chin_tools::wrapper::anyhow::AResult;

use crate::model::{
    db::{Article, BlockedLink, Feed, FeedLog, WebCache},
    dto::{
        ArticleFilterReq, CollectionMeta, FeedSortReq, FeedsCountReq, MarkAllReadReq,
        SearchArticleRsp, SearchReq,
    },
};

#[async_trait]
pub trait Mapper: ArticleMapper + FeedMapper + Send + Sync {
    async fn global_search(&self, search: SearchReq) -> AResult<Vec<SearchArticleRsp>>;

    async fn get_web_cache(&self, url: &str) -> AResult<WebCache>;
    async fn set_web_cache(&self, url: &WebCache) -> AResult<u64>;

    async fn add_blocked_domain(&self, url: &str) -> AResult<u64>;
    async fn fetch_blocked_domains(&self) -> AResult<Vec<BlockedLink>>;

    async fn fix_db(&self) -> AResult<()>;
}

#[async_trait]
pub trait ArticleMapper {
    async fn get_article_with_uuid(&self, uuid: &str) -> AResult<Article>;
    async fn get_article_with_url(&self, url: &str) -> AResult<Article>;

    async fn set_article_content_with_url(&self, url: &str, content: &str) -> AResult<()>;

    async fn update_article_read_status(&self, uuid: &str, is_read: bool) -> AResult<usize>;

    async fn update_article_star_status(&self, uuid: &str, is_starred: bool) -> AResult<usize>;
    async fn get_articles(&self, filter: &ArticleFilterReq) -> AResult<Vec<Article>>;

    async fn add_articles(&self, articles: &Vec<Article>) -> AResult<u64>;
}

#[async_trait]
pub trait FeedMapper {
    async fn get_sub_ids(&self, id: &str) -> AResult<Vec<String>>;
    async fn get_sub_ids_as_sql_seg(&self, id: &str) -> AResult<String> {
        let inner = self
            .get_sub_ids(&id)
            .await?
            .into_iter()
            .map(|e| format!("'{}'", e))
            .collect::<Vec<String>>()
            .join(",");
        Ok(format!("({})", inner))
    }

    async fn mark_as_read(&self, id: &MarkAllReadReq) -> AResult<()>;

    async fn get_sub_true_feeds(&self, id: &str) -> AResult<Vec<Feed>>;

    async fn get_all_feeds(&self) -> AResult<Vec<Feed>>;

    async fn write_log(&self, log: &FeedLog) -> AResult<()>;
    async fn get_feed_logs(&self, id: &str) -> AResult<Vec<FeedLog>>;
    async fn get_last_success_feed_log(&self, id: &str) -> Option<FeedLog>;

    async fn delete_feed(&self, id: &str) -> AResult<u64>;
    async fn update_feed_sort(&self, sorts: &Vec<FeedSortReq>) -> AResult<u64>;
    async fn add_feed(&self, feed: &Feed) -> AResult<()>;

    async fn count_per_feed(&self, req: &FeedsCountReq) -> AResult<HashMap<String, i64>>;

    async fn get_collection_metas(&self) -> AResult<CollectionMeta>;

    async fn update_folder_name(&self, uuid: &str, name: &str) -> AResult<()>;
    async fn update_feed_sync_interval(&self, id: &str, interval: u32) -> AResult<u64>;

    async fn move_channel_into_folder(&self, sub_id: &str, parent_id: &str) -> AResult<()>;
}
