use chrono::{DateTime, FixedOffset, Local};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

use crate::mapper::feed_rs::ring_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    id: String,
    pub ring_id: String,
    pub pub_id: Option<String>,
    pub title: String,
    pub link: String,
    pub feed_id: String,
    pub feed_title: String,
    pub description: String,
    pub author: String,
    pub pub_time: DateTime<FixedOffset>,
    pub content: String,
    pub cached_content: Option<String>,
    pub create_time: DateTime<FixedOffset>,
    pub update_time: DateTime<FixedOffset>,
    pub is_read: bool,
    pub is_starred: bool,
}

impl Article {
    pub fn new(
        pub_id: Option<String>,
        title: String,
        link: String,
        feed_id: String,
        feed_title: String,
        description: String,
        author: String,
        pub_time: DateTime<FixedOffset>,
        content: String,
        cached_content: Option<String>,
        create_time: DateTime<FixedOffset>,
        update_time: DateTime<FixedOffset>,
        is_read: bool,
        is_starred: bool,
        feed_url: &str,
    ) -> Self {
        let id = uuid::Uuid::new_v4().hyphenated().to_string();
        let ring_id = ring_id(pub_id.as_ref(), feed_url);
        Article {
            id,
            ring_id,
            pub_id,
            title,
            link,
            feed_id,
            description,
            author,
            pub_time,
            content,
            cached_content,
            create_time,
            update_time,
            is_read,
            is_starred,
            feed_title,
        }
    }

    pub fn new_from_db(
        id: String,
        ring_id: String,
        pub_id: Option<String>,
        title: String,
        link: String,
        feed_id: String,
        feed_title: String,
        description: String,
        author: String,
        pub_time: DateTime<FixedOffset>,
        content: String,
        cached_content: Option<String>,
        create_time: DateTime<FixedOffset>,
        update_time: DateTime<FixedOffset>,
        is_read: bool,
        is_starred: bool,
    ) -> Self {
        Article {
            id,
            ring_id,
            pub_id,
            title,
            link,
            feed_id,
            feed_title,
            description,
            author,
            pub_time,
            content,
            cached_content,
            create_time,
            update_time,
            is_read,
            is_starred,
        }
    }

    pub fn get_id(&self) -> &str {
        return &self.id;
    }
}

#[derive(Debug, Deserialize, EnumString, AsRefStr, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum FeedTypeMark {
    #[strum(serialize = "channel")]
    Feed,
    #[strum(serialize = "folder")]
    Folder,
}

impl Serialize for FeedTypeMark {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            FeedTypeMark::Feed => serializer.serialize_str("channel"),
            FeedTypeMark::Folder => serializer.serialize_str("folder"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub link: String,

    pub item_type: FeedTypeMark,
    pub logo: String,
    pub feed_url: String,
    pub description: String,
    pub create_time: DateTime<FixedOffset>,
    pub update_time: DateTime<FixedOffset>,
    pub sort: i32,

    pub sync_interval_sec: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeedLog {
    pub feed_id: String,
    pub last_pub_date: Option<DateTime<FixedOffset>>,
    pub healthy: bool,
    pub log: String,
    pub create_time: DateTime<FixedOffset>,
    pub update_time: DateTime<FixedOffset>,
}

impl Default for FeedLog {
    fn default() -> Self {
        Self {
            feed_id: Default::default(),
            last_pub_date: Default::default(),
            healthy: true,
            log: Default::default(),
            create_time: Local::now().fixed_offset(),
            update_time: Local::now().fixed_offset(),
        }
    }
}

#[derive(Serialize)]
pub struct WebCache {
    pub id: String,
    pub url: String,
    pub content_type: String,
}

#[derive(Serialize)]
pub struct BlockedLink {
    pub url: String,
    pub insert_time: DateTime<FixedOffset>,
    pub update_time: DateTime<FixedOffset>,
}
