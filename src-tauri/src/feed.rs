use std::collections::HashMap;

use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::db;
use crate::folder;
use crate::models;
use crate::schema;

pub fn delete_channel(uuid: String) -> usize {
  let mut connection = db::establish_connection();
  let channel = schema::channels::dsl::channels
    .filter(schema::channels::uuid.eq(&uuid))
    .load::<models::Channel>(&mut connection)
    .expect("Expect find channel");

  if channel.len() == 1 {
    let result =
      diesel::delete(schema::channels::dsl::channels.filter(schema::channels::uuid.eq(&uuid)))
        .execute(&mut connection)
        .expect("Expect delete channel");

    diesel::delete(
      schema::articles::dsl::articles.filter(schema::articles::channel_uuid.eq(&uuid)),
    )
    .execute(&mut connection)
    .expect("Expect delete channel");

    diesel::delete(
      schema::feed_metas::dsl::feed_metas.filter(schema::feed_metas::child_uuid.eq(&uuid)),
    )
    .execute(&mut connection)
    .expect("Expect delete channel");

    return result;
  } else {
    return 0;
  }
}

pub fn batch_delete_channel(channel_uuids: Vec<String>) -> usize {
  let mut connection = db::establish_connection();
  let result = diesel::delete(
    schema::channels::dsl::channels.filter(schema::channels::uuid.eq_any(&channel_uuids)),
  )
  .execute(&mut connection)
  .expect("Expect delete channel");

  diesel::delete(
    schema::articles::dsl::articles.filter(schema::articles::channel_uuid.eq_any(&channel_uuids)),
  )
  .execute(&mut connection)
  .expect("Expect delete channel");

  result
}

pub fn get_feed_meta_with_uuids(channel_uuids: Vec<String>) -> Vec<models::FeedMeta> {
  let mut connection = db::establish_connection();
  let result = schema::feed_metas::dsl::feed_metas
    .filter(schema::feed_metas::child_uuid.eq_any(&channel_uuids))
    .load::<models::FeedMeta>(&mut connection)
    .expect("Expect get feed meta");

  result
}

pub fn get_all_feed_meta() -> Vec<models::FeedMeta> {
  let mut connection = db::establish_connection();
  let result = schema::feed_metas::dsl::feed_metas
    .order(schema::feed_metas::sort.desc())
    .load::<models::FeedMeta>(&mut connection)
    .expect("Expect get feed meta");

  result
}

#[derive(Debug, Queryable, Serialize, QueryableByName)]
pub struct UnreadTotal {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub channel_uuid: String,
  #[diesel(sql_type = diesel::sql_types::Integer)]
  pub unread_count: i32,
}

#[derive(Debug, Queryable, Serialize, QueryableByName)]
pub struct MetaGroup {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub child_uuid: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub parent_uuid: String,
  #[diesel(sql_type = diesel::sql_types::Integer)]
  pub sort: i32,
}

pub fn get_unread_total() -> HashMap<String, i32> {
  const SQL_QUERY_UNREAD_TOTAL: &str = "
    SELECT id, channel_uuid, count(read_status) as unread_count FROM articles WHERE  read_status = 1 group by channel_uuid;
  ";
  let sql_folders: &str = "
    select child_uuid, parent_uuid, sort from feed_metas;
  ";

  let mut connection = db::establish_connection();
  let record = diesel::sql_query(SQL_QUERY_UNREAD_TOTAL)
    .load::<UnreadTotal>(&mut connection)
    .unwrap_or(vec![]);
  let total_map = record.into_iter()
  .map(|r| (r.channel_uuid.clone(), r.unread_count.clone()))
  .collect::<HashMap<String, i32>>();
  let meta_group = diesel::sql_query(sql_folders)
    .load::<MetaGroup>(&mut connection)
    .unwrap_or(vec![]);
  let mut result_map: HashMap<String, i32> = HashMap::new();

  for group in meta_group {
    match total_map.get(&group.child_uuid) {
      Some(count) => {
        if group.parent_uuid == "".to_string() {
          result_map.entry(group.child_uuid)
            .or_insert(count.clone());
        } else {
          println!("{:?}", group.parent_uuid);
          let c= result_map.entry(group.parent_uuid)
            .or_insert(0);
          *c += count;
        }
      },
      None => {}
    };
  }

  result_map
}

#[derive(Deserialize)]
pub struct FeedMetaUpdateRequest {
  pub parent_uuid: String,
  pub sort: i32,
}

pub fn update_feed_meta(uuid: String, update: FeedMetaUpdateRequest) -> usize {
  let mut connection = db::establish_connection();
  let updated_row = diesel::update(
    schema::feed_metas::dsl::feed_metas.filter(schema::feed_metas::child_uuid.eq(uuid)),
  )
  .set((
    schema::feed_metas::parent_uuid.eq(update.parent_uuid),
    schema::feed_metas::sort.eq(update.sort),
  ))
  .execute(&mut connection)
  .expect("update feed meta");

  updated_row
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FeedItem {
  pub item_type: String,
  pub uuid: String,
  pub title: String,
  pub sort: i32,
  pub child_uuids: Option<Vec<String>>,
  pub link: Option<String>,
}

#[derive(Debug, Queryable, Serialize, QueryableByName)]
pub struct FolderJoinItem {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub title: String,
  #[diesel(sql_type = diesel::sql_types::Integer)]
  pub sort: i32,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub uuid: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub link: String
}

pub fn get_feeds() -> Vec<FeedItem> {
  let sql_folder = "
  SELECT C.name AS title, F.child_uuid AS uuid, F.sort, F.parent_uuid as link FROM folders AS C LEFT JOIN feed_metas AS F where F.parent_uuid = '' AND C.uuid = F.child_uuid;
  ";
  let sql_channel = "
  SELECT C.title AS title, F.child_uuid AS uuid, F.sort, C.link FROM channels as C LEFT JOIN  feed_metas AS F where F.parent_uuid = '' and C.uuid = F.child_uuid;";

  let mut connection = db::establish_connection();

  let channels = diesel::sql_query(sql_channel)
  .load::<FolderJoinItem>(&mut connection)
  .unwrap_or(vec![]);

  let folders = diesel::sql_query(sql_folder)
  .load::<FolderJoinItem>(&mut connection)
  .unwrap_or(vec![]);

  let mut result: Vec<FeedItem> = Vec::new();

  for folder in folders {
    result.push(FeedItem {
      item_type: String::from("folder"),
      uuid: folder.uuid,
      title: folder.title,
      sort: folder.sort,
      link: Some(folder.link),
      child_uuids: Some(vec![]),
    })
  }

  for channel in channels {
    result.push(FeedItem {
      item_type: String::from("channel"),
      uuid: channel.uuid,
      title: channel.title,
      sort: channel.sort,
      link: Some(channel.link),
      child_uuids: Some(Vec::new()),
    })
  }

  result.sort_by(|a, b| a.sort.cmp(&b.sort));

  result
}

pub fn get_last_sort(connection: &mut diesel::SqliteConnection) -> i32 {
  let last_sort = schema::feed_metas::dsl::feed_metas
    .select(schema::feed_metas::sort)
    .filter(schema::feed_metas::dsl::parent_uuid.is(""))
    .get_results::<i32>(connection);

  let last_sort = match last_sort {
    Ok(mut rec) => rec.pop(),
    Err(_) => None,
  };

  let last_sort = match last_sort {
    Some(s) => s,
    None => 0,
  };

  last_sort
}

pub fn add_channel(channel: models::NewChannel, articles: Vec<models::NewArticle>) -> usize {
  let mut connection = db::establish_connection();
  let result = diesel::insert_or_ignore_into(schema::channels::dsl::channels)
    .values(&channel)
    .execute(&mut connection);
  let result = match result {
    Ok(r) => {
      if r == 1 {
        let last_sort = get_last_sort(&mut connection);
        let meta_record = models::NewFeedMeta {
          child_uuid: String::from(channel.uuid),
          parent_uuid: "".to_string(),
          sort: last_sort + 1,
        };

        diesel::insert_or_ignore_into(schema::feed_metas::dsl::feed_metas)
          .values(meta_record)
          .execute(&mut connection)
          .expect("Expect create feed meta");
      }

      r
    }
    Err(_) => 0,
  };

  println!(" new result {:?}", result);

  if result == 1 {
    println!("start insert articles");

    let articles = diesel::insert_or_ignore_into(schema::articles::dsl::articles)
      .values(articles)
      .execute(&mut connection);

    println!("articles {:?}", articles);
  }

  result
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedSort {
  uuid: String,
  item_type: String,
  sort: i32,
}

pub fn update_feed_sort(sorts: Vec<FeedSort>) -> usize {
  let mut connection = db::establish_connection();

  for item in sorts {
    let result = diesel::update(
      schema::feed_metas::dsl::feed_metas.filter(schema::feed_metas::child_uuid.eq(&item.uuid)),
    )
    .set(schema::feed_metas::sort.eq(item.sort))
    .execute(&mut connection)
    .expect("msg");
    println!(" update channel{:?} {:?}", &item.uuid, item.sort);
  }

  1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelFilter {
  pub parent_uuid: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChannelQueryResult {
  list: Vec<models::Channel>,
  // pub count: i32,
}

pub fn get_channels(filter: ChannelFilter) -> ChannelQueryResult {
  let mut connection = db::establish_connection();
  let mut query = schema::channels::dsl::channels.into_boxed();

  match filter.parent_uuid {
    Some(parent_uuid) => {
      let relations = schema::feed_metas::dsl::feed_metas
        .filter(schema::feed_metas::parent_uuid.eq(parent_uuid))
        .load::<models::FeedMeta>(&mut connection)
        .expect("Expect get feed meta");
      let mut folder_uuids: Vec<String> = vec![];
      let mut channel_uuids: Vec<String> = vec![];

      for relation in relations {
        if relation.parent_uuid == "" {
          folder_uuids.push(relation.parent_uuid);
        } else {
          channel_uuids.push(relation.child_uuid);
        }
      }

      query = query.filter(schema::channels::uuid.eq_any(channel_uuids));
    }
    None => {
      1;
    }
  }

  query = query.order(schema::channels::dsl::create_date.desc());

  let result = query
    .load::<models::Channel>(&mut connection)
    .expect("Expect loading articles");

  ChannelQueryResult { list: result }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_feeds() {
    let result = get_feeds();
    println!("{:?}", result)
  }

  #[test]
  fn test_get_unread_total() {
    let record = get_unread_total();

    println!("{:?}", record);
  }
}
