use std::path::PathBuf;

use axum::body::Body;
use futures::TryStreamExt;
use tokio::time;
use uuid::Uuid;

use crate::{
    mapper::feed_rs::{map_entry_to_article, map_feed_to_feed},
    model::{
        alias::RResult,
        db::{Feed, FeedTypeMark, WebCache},
        dto::{FeedAddRsp, FeedSyncResult, ForwardUrlReq, MarkAllUnreadReq},
    },
};

use chrono::DateTime;
use tracing::{error, info};

use crate::{
    controller::WebAppState,
    model::{
        db::{Article, FeedLog},
        dto::FeedAddReq,
    },
};

impl WebAppState {
    pub async fn fetch_article_and_cache(&self, req: &ForwardUrlReq) -> RResult<String> {
        let url = req.url.as_str();
        let art = self.pool.get_article_with_url(url).await?;

        if art.cached_content.is_some() && art.cached_content.as_ref().unwrap().len() > 0 {
            Ok(art.cached_content.unwrap())
        } else {
            let res = self.page_scraper.fetch_page(&url).await?;
            self.pool.set_article_content_with_url(&url, &res).await?;
            Ok(res)
        }
    }

    /// https://stackoverflow.com/questions/76383850/axum-send-image-as-response
    pub async fn fetch_attachment_and_cache(
        &self,
        req: &ForwardUrlReq,
    ) -> RResult<(WebCache, Body)> {
        let url = req.url.as_str().replace("_", "/");
        let url = String::from_utf8(base64::decode(url)?)?;
        let art = self.pool.get_web_cache(url.as_str()).await;
        let md5 = format!("{:x}", md5::compute(&req.url));

        let file = chin_tools::utils::pathutils::join_from_path(
            &PathBuf::from(&self.config.common.asset_base_dir),
            &md5.as_str(),
        );

        let parent = file
            .parent()
            .ok_or(anyhow::anyhow!("unable to find parent"))?;
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let cache = match art {
            Ok(cache) => cache,
            Err(_) => {
                let (stream, ct) = self.page_scraper.get_attachment(url.as_str()).await?;

                let cache = WebCache {
                    id: md5,
                    url: url.to_string(),
                    content_type: ct,
                };

                chin_tools::utils::fileutils::stream_to_file_async(
                    stream.map_err(|e| anyhow::anyhow!(e)),
                    &file,
                )
                .await?;
                self.pool.set_web_cache(&cache).await?;

                cache
            }
        };

        let v = chin_tools::utils::fileutils::file_to_stream_async(&file).await?;

        Ok((cache, Body::from_stream(v)))
    }

    pub fn sync_feeds_at_interval(&self) {
        let state = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                match state.sync_feeds(None).await {
                    Ok(_) => {
                        info!("sync feeds");
                    }
                    Err(err) => {
                        error!("occured error when syncing: {}", err)
                    }
                }
            }
        });
    }

    pub async fn sync_feeds(&self, id: Option<&str>) -> RResult<Vec<FeedSyncResult>> {
        let mut result = vec![];
        let res = match id {
            Some(id) => self.pool.get_sub_true_feeds(&id).await?,
            None => self.pool.get_all_feeds().await?,
        };

        for feed in res {
            let app_state = self.clone();
            let id = feed.id.clone();
            let mut feed_log: FeedLog = FeedLog::default();
            feed_log.feed_id = id.to_owned();

            let parsed = app_state.page_scraper.parse_feed(&feed.feed_url).await;
            match parsed {
                Ok(rfeed) => {
                    if let Some(v) = rfeed.updated.as_ref() {
                        feed_log.last_pub_date.replace(v.fixed_offset());
                    };
                    let articles: Vec<Article> = rfeed
                        .entries
                        .iter()
                        .map(|e| map_entry_to_article(e, &feed.id))
                        .collect();
                    info!("fetch {} articles from {}", articles.len(), feed.feed_url);
                    let count = app_state.pool.add_articles(&articles).await?;
                    let title: String =
                        if let Some(title) = rfeed.title.map(|e| e.content.as_str().to_owned()) {
                            title
                        } else {
                            feed.feed_url.clone()
                        };
                    result.push(FeedSyncResult {
                        feed_id: id.clone(),
                        title: title,
                        count: count,
                        err_info: "".to_string(),
                    })
                }
                Err(err) => {
                    feed_log.log = err.to_string();
                    feed_log.healthy = false;
                    result.push(FeedSyncResult {
                        feed_id: id.clone(),
                        title: feed.title.clone(),
                        count: 0,
                        err_info: err.to_string(),
                    })
                }
            };
            match app_state.pool.write_log(&feed_log).await {
                Ok(_o) => {}
                Err(err) => {
                    error!("unable to write log: {}", err)
                }
            }
        }

        Ok(result)
    }

    pub async fn add_feed(&self, req: FeedAddReq) -> RResult<FeedAddRsp> {
        let url = req.url;

        let res = self.page_scraper.parse_feed(&url).await?;

        let channel_uuid = Uuid::new_v4().hyphenated().to_string();
        let feed = map_feed_to_feed(&channel_uuid, None, &0, &url, &res);
        let articles = res
            .entries
            .iter()
            .map(|e| map_entry_to_article(e, &feed.id))
            .collect();

        info!("add feed, {:?}", feed);
        self.pool.add_feed(&feed).await?;
        info!("add articles, ");
        let article_count = self.pool.add_articles(&articles).await?;

        Ok(FeedAddRsp {
            feed,
            article_count,
        })
    }

    pub async fn mark_as_read(&self, req: &MarkAllUnreadReq) -> RResult<()> {
        self.pool.mark_as_read(&req).await?;

        Ok(())
    }

    pub(crate) async fn add_folder(&self, req: &String) -> RResult<Feed> {
        let channel_uuid = Uuid::new_v4().hyphenated().to_string();
        let folder_url = format!("folder-{}", req);
        let feed = Feed {
            id: channel_uuid.clone(),
            parent_id: None,
            title: req.clone(),
            link: folder_url.clone(),
            item_type: FeedTypeMark::Folder,
            logo: "".to_string(),
            feed_url: folder_url.clone(),
            description: "".to_string(),
            create_time: DateTime::from_timestamp_micros(0).unwrap().fixed_offset(),
            update_time: DateTime::from_timestamp_micros(0).unwrap().fixed_offset(),
            sort: 0,
        };

        self.pool.add_feed(&feed).await?;
        Ok(feed)
    }
}
