use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::anyhow;
use axum::http::HeaderValue;
use bytes::Bytes;
use chin_tools::wrapper::anyhow::AResult;
use feed_rs::parser;
use reqwest::{self, header, Response};
use scraper::{self};
use serde::Serialize;
use tracing::{error, info};

use crate::log_and_bail;
use crate::model::ProxyConfig;
use crate::tool::request::create_client;

use super::db::Mapper;

#[derive(Debug, Default, Serialize)]
pub struct PageScraperResult {
    pub title: String,
    pub content: String,
    pub author: String,
    pub date_published: String,
    pub domain: String,
    pub url: String,
    pub excerpt: String,
}

#[derive(Clone)]
pub struct PageScraper {
    proxy: Option<ProxyConfig>,
    proxy_urls: Arc<RwLock<HashSet<String>>>,
    db_mapper: Arc<dyn Mapper>,
}

impl PageScraper {
    pub fn new(proxy: Option<&ProxyConfig>, mapper: &Arc<dyn Mapper>) -> Self {
        Self {
            proxy: proxy.map(|e| e.clone()).clone(),
            proxy_urls: Arc::new(RwLock::new(HashSet::new())),
            db_mapper: mapper.clone(),
        }
    }

    pub async fn init(&self) -> AResult<()> {
        let urls = self.db_mapper.fetch_blocked_domains().await?;
        let _ = self.proxy_urls.write().map(|mut lock| {
            let urls = urls.into_iter().map(|e| e.url);
            lock.extend(urls)
        });

/*         if let Err(err) = self.db_mapper.fix_db().await {
            error!("unable to fix db: {}", err);
        } */

        Ok(())
    }

    pub async fn request(&self, url: &str) -> AResult<Response> {
        let parsed = url::Url::parse(url)?;
        let host = parsed.host_str();
        let proxy = if host.map_or(false, |url| {
            self.proxy_urls
                .read()
                .map_or(false, |set| set.contains(url))
        }) {
            self.proxy.as_ref()
        } else {
            None
        };

        match create_client(proxy)
            .get(url)
            .timeout(Duration::from_secs(35))
            .send()
            .await
        {
            Ok(w) => {
                return Ok(w);
            }
            Err(err) => {
                error!("unable to request: {}", err);
            }
        }

        if proxy.is_none() && self.proxy.as_ref().is_some() {
            info!("fallback to use proxy");
            if let Ok(w) = create_client(self.proxy.as_ref()).get(url).send().await {
                if let Some(host) = host {
                    if let Ok(mut lock) = self.proxy_urls.write() {
                        lock.insert(host.to_string());
                    }
                    self.db_mapper.add_blocked_domain(host).await?;
                };
                return Ok(w);
            }
        }

        anyhow::bail!("unable to get response from {}", url)
    }

    pub async fn fetch_page(&self, url: &str) -> AResult<String> {
        let response = self.request(url).await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let content = response.text().await.unwrap();

                Ok(content)
            }
            e => log_and_bail!("wrong status code: {}", e),
        }
    }

    pub async fn get_first_image_or_og_image(&self, url: &str) -> AResult<String> {
        let res = self.request(url).await?;
        let body = res.text().await?;
        let document = scraper::Html::parse_document(&body);

        let og_selector = scraper::Selector::parse(r#"meta[property="og:image"]"#)
            .map_err(|e| anyhow!("{}", e.to_string()))?;
        if let Some(og_image) = document.select(&og_selector).next() {
            if let Some(content) = og_image.value().attr("content") {
                return Ok(content.to_string());
            }
        }

        let img_selector = scraper::Selector::parse("img").unwrap();
        for img in document.select(&img_selector) {
            if let Some(src) = img.value().attr("src") {
                return Ok(src.to_string());
            }
        }

        log_and_bail!("unable to get_first_image_or_og_image")
    }

    pub async fn parse_feed(&self, url: &str) -> AResult<feed_rs::model::Feed> {
        info!("begin to parse feed: {}", url);
        match self.request(url).await {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => match response.text().await {
                    Ok(content) => {
                        let c: AResult<feed_rs::model::Feed> =
                            match parser::parse(content.as_bytes()) {
                                Ok(feed) => Ok(feed),
                                Err(error) => Err(anyhow::anyhow!("unable to parse: {:?}", error)),
                            };
                        c
                    }
                    Err(error) => Err(anyhow::anyhow!(
                        "cannot extract text from response: {:?}",
                        error
                    )),
                },
                reqwest::StatusCode::NOT_FOUND => Err(anyhow::anyhow!(
                    "Could not find a feed at the location, {}",
                    url
                )),

                other => Err(anyhow::anyhow!("unable to parse: {}", other)),
            },
            Err(err) => Err(anyhow::anyhow!("unable to parse: {:?}", err)),
        }
    }

    pub async fn get_attachment(
        &self,
        url: &str,
    ) -> AResult<(
        impl futures_core::Stream<Item = reqwest::Result<Bytes>>,
        String,
    )> {
        let rsp = self.request(url).await?;

        let header = rsp.headers();
        let content_type = header
            .get(header::CONTENT_TYPE)
            .unwrap_or(&HeaderValue::from_static("text/html"))
            .to_str()?
            .to_string();
        let bytes = rsp.bytes_stream();

        Ok((bytes, content_type))
    }
}
