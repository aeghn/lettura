use crate::model::db::{Article, Feed, FeedTypeMark};
use chrono::{DateTime, Local};
use feed_rs::model::Entry;

pub fn map_entry_to_article(entry: &Entry, feed: &Feed) -> Article {
    let title = match &entry.title {
        Some(link) => link.content.to_string(),
        None => String::from(""),
    };

    let link = match entry.links.get(0) {
        Some(link) => link.href.to_string(),
        None => String::from(""),
    };

    // A short summary of the item
    let description = match &entry.summary {
        Some(summary) => summary.content.clone(),
        None => String::from(""),
    };

    // The content of the item
    let content = match &entry.content {
        Some(content) => content.body.clone().unwrap_or(String::from("")),
        None => String::from(""),
    };

    // Time at which this item was first published
    let pub_date = match entry.published {
        Some(t) => t.fixed_offset(),
        None => DateTime::from_timestamp_micros(0).unwrap().fixed_offset(),
    };

    // Authors of this item
    let author = entry
        .authors
        .iter()
        .map(|e| e.name.clone())
        .collect::<Vec<String>>()
        .join("; ");

    Article::new(
        Some(entry.id.clone()),
        title,
        link,
        feed.id.clone(),
        feed.title.clone(),
        description,
        author,
        pub_date,
        content,
        None,
        Local::now().fixed_offset(),
        Local::now().fixed_offset(),
        false,
        false,
        &feed.feed_url,
    )
}

pub fn map_feed_to_feed(
    id: &str,
    parent_id: Option<String>,
    sort: &i32,
    url: &str,
    res: &feed_rs::model::Feed,
) -> Feed {
    let title = match &res.title {
        Some(link) => link.content.to_string(),
        None => String::from(""),
    };

    let link = match res.links.get(0) {
        Some(link) => link.href.to_string(),
        None => String::from(""),
    };

    let description = match &res.description {
        Some(title) => title.content.clone(),
        None => String::from(""),
    };

    let logo = match &res.logo {
        Some(t) => t.uri.clone(),
        None => String::from(""),
    };

    return Feed {
        id: id.to_string(),
        parent_id: None,
        title,
        link,
        item_type: FeedTypeMark::Feed,
        logo,
        feed_url: url.to_owned(),
        description,
        create_time: DateTime::from_timestamp_micros(0).unwrap().fixed_offset(),
        update_time: DateTime::from_timestamp_micros(0).unwrap().fixed_offset(),
        sort: sort.clone(),
        sync_interval_sec: 86400,
    };
}
