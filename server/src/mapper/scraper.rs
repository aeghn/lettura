use anyhow::anyhow;
use axum::http::HeaderValue;
use bytes::Bytes;
use feed_rs::parser;
use reqwest::{self, header, Client};
use scraper::{self};
use serde::Serialize;
use tracing::info;

use crate::log_and_bail;
use crate::model::alias::RResult;
use crate::model::ProxyConfig;
use crate::tool::request::create_client;

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
}

impl PageScraper {
    pub fn new(proxy: Option<&ProxyConfig>) -> Self {
        Self {
            proxy: proxy.map(|e| e.clone()).clone(),
        }
    }

    pub fn get_client(&self) -> Client {
        create_client(self.proxy.as_ref())
    }

    pub fn get_client_with_url(&self, url: &str) -> Client {
        if url.contains(".cn") {
            create_client(None)
        } else {
            create_client(self.proxy.as_ref())
        }
    }

    pub async fn fetch_page(&self, url: &str) -> RResult<String> {
        let response = self.get_client().get(url).send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let content = response.text().await.unwrap();

                Ok(content)
            }
            e => log_and_bail!("wrong status code: {}", e),
        }
    }

    pub async fn get_first_image_or_og_image(&self, url: &str) -> RResult<String> {
        let res = self.get_client().get(url).send().await?;
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

    pub async fn parse_feed(&self, url: &str) -> RResult<feed_rs::model::Feed> {
        info!("begin to parse feed: {}", url);
        match self.get_client_with_url(url).get(url).send().await {
            Ok(response) => match response.status() {
                reqwest::StatusCode::OK => match response.text().await {
                    Ok(content) => {
                        let c: RResult<feed_rs::model::Feed> =
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
    ) -> RResult<(
        impl futures_core::Stream<Item = reqwest::Result<Bytes>>,
        String,
    )> {
        let rsp = self.get_client().get(url).send().await?;

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
