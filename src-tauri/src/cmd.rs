use reqwest;
use serde::{Deserialize, Serialize};
use tauri::command;
use uuid::Uuid;

use crate::db;
use crate::models;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse<T> {
  status: i32,
  data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedRes {
  channel: rss::Channel,
  items: Vec<rss::Item>,
}

pub async fn fetch_rss_item(url: &str) -> Option<rss::Channel> {
  let response = reqwest::get(url).await.unwrap();

  println!("{}", &response.status());

  match response.status() {
    reqwest::StatusCode::OK => {
      let content = response.bytes().await.unwrap();
      let channel = rss::Channel::read_from(&content[..]).unwrap();

      Some(channel)
    }
    _ => {
      println!("ddddd");
      None
    }
  }
}

#[command]
pub async fn fetch_feed(url: String) -> Option<rss::Channel> {
  let res = fetch_rss_item(&url).await;

  res
}

#[command]
pub async fn get_channels() -> Vec<models::Channel> {
  let results = db::get_channels();

  return results;
}

pub fn create_channel_model(uuid: &String, url: &String, res: &rss::Channel) -> models::NewChannel {
  let image = match &res.image {
    Some(t) => String::from(&t.url),
    None => String::from(""),
  };
  let date = match &res.pub_date {
    Some(t) => String::from(t),
    None => String::from(""),
  };
  let channel = models::NewChannel {
    uuid: uuid.to_string(),
    title: res.title.to_string(),
    link: res.link.to_string(),
    image: image.to_string(),
    feed_url: url.to_string(),
    description: res.description.to_string(),
    pub_date: date,
  };

  return channel;
}

pub fn create_article_models(channel_uuid: &String, feed_url: &String, res: &rss::Channel) -> Vec<models::NewArticle> {
  let mut articles: Vec<models::NewArticle> = Vec::new();

  for item in res.items() {
    let article_uuid = Uuid::new_v4().hyphenated().to_string();
    let title = item.title.clone().unwrap_or(String::from("no title"));
    let link = item.link.clone().unwrap_or(String::from("no link"));
    let content = item.content.clone().unwrap_or(String::from("no content"));
    let description = item
      .description
      .clone()
      .unwrap_or(String::from("no description"));
    let date = String::from(item.pub_date().clone().unwrap());

    let s = models::NewArticle {
      uuid: article_uuid,
      channel_uuid: channel_uuid.to_string(),
      title,
      link,
      content,
      feed_url: feed_url.to_string(),
      description,
      pub_date: date,
    };

    articles.push(s);
  }

  articles
}

#[command]
pub async fn add_channel(url: String) -> usize {
  println!("request channel {}", &url);

  let res = fetch_rss_item(&url).await.unwrap();
  let channel_uuid = Uuid::new_v4().hyphenated().to_string();

  println!("request channel {:?}", &res);

  let channel = create_channel_model(&channel_uuid, &url, &res);
  let articles = create_article_models(&channel_uuid, &url, &res);
  let res = db::add_channel(channel, articles);

  res
}

#[command]
pub fn get_articles(uuid: String) -> db::ArticleQueryResult {
  println!("get articles from rust");
  let res = db::get_article(db::ArticleFilter {
    channel_uuid: Some(uuid),
  });

  res
}

#[command]
pub fn delete_channel(uuid: String) -> usize {
  let result = db::delete_channel(uuid);

  result
}

#[command]
pub async fn sync_articles_with_channel_uuid(uuid: String) -> usize {
  let channel = db::get_channel_by_uuid(uuid);
  match channel {
    Some(channel) => {
      let res = fetch_rss_item(&channel.feed_url).await.unwrap();

      let mut articles: Vec<models::NewArticle> = Vec::new();

      for item in res.items() {
        let article_uuid = Uuid::new_v4().hyphenated().to_string();
        let title = item.title.clone().unwrap_or(String::from("no title"));
        let link = item.link.clone().unwrap_or(String::from("no link"));
        let content = item.content.clone().unwrap_or(String::from("no content"));
        let description = item
          .description
          .clone()
          .unwrap_or(String::from("no description"));
        let date = String::from(item.pub_date().clone().unwrap());

        let s = models::NewArticle {
          uuid: article_uuid,
          channel_uuid: String::from(&channel.uuid),
          title,
          link,
          content,
          feed_url: String::from(&channel.feed_url),
          description,
          pub_date: date,
        };

        articles.push(s);
      }

      let result = db::add_articles(String::from(&channel.uuid), articles);

      result
    }
    None => 0,
  }
}


#[command]
pub async fn import_channels(list: Vec<String> ) -> usize {
  println!("{:?}", &list);
  for url in &list {
    add_channel(url.to_string()).await;
  }
  1
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_articles() {
    let uuid = String::from("9a6ca3f0-41f2-4486-a50a-1a41f1e80b56");
    get_articles(uuid);
  }

  #[test]
  fn test_delete_channel() {
    let url = "9a6ca3f0-41f2-4486-a50a-1a41f1e80b56";
    delete_channel(String::from(url));
  }
}
