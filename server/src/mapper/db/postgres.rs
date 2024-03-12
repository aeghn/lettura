use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use async_trait::async_trait;
use bytes::Bytes;
use deadpool_postgres::{
    tokio_postgres::{self, Row},
    Pool,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::model::{
    alias::RResult,
    db::{Article, Feed, FeedLog, FeedTypeMark, WebCache},
    dto::{
        ArticleFilterReq, CollectionMeta, FeedSortReq, FeedsCountReq, MarkAllReadReq,
        SearchArtileRsp, SearchReq,
    },
};

use super::{ArticleMapper, FeedMapper, Mapper};

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    user: String,
    pass: String,
    dbname: String,
    host: String,
    port: u16,
}

impl Into<deadpool_postgres::Config> for PostgresConfig {
    fn into(self) -> deadpool_postgres::Config {
        let mut cfg = deadpool_postgres::Config::new();
        cfg.user = Some(self.user);
        cfg.password = Some(self.pass);
        cfg.dbname = Some(self.dbname);
        cfg.host = Some(self.host);
        cfg.port = Some(self.port);
        cfg
    }
}

pub struct PostgresMapper {
    pub pool: Pool,
}

impl PostgresMapper {
    pub fn new(config: PostgresConfig) -> anyhow::Result<PostgresMapper> {
        let pool = Into::<deadpool_postgres::Config>::into(config)
            .create_pool(None, tokio_postgres::NoTls)?;

        Ok(PostgresMapper { pool })
    }

    fn map_row_to_article(row: &Row) -> RResult<Article> {
        Ok(Article::new_from_db(
            row.try_get("id")?,
            row.try_get("pub_id")?,
            row.try_get("title")?,
            row.try_get("link")?,
            row.try_get("feed_id")?,
            row.try_get("feed_title")?,
            row.try_get("description")?,
            row.try_get("author")?,
            row.try_get("pub_time")?,
            row.try_get("content")?,
            row.try_get("cached_content")?,
            row.try_get("create_time")?,
            row.try_get("update_time")?,
            row.try_get("is_read")?,
            row.try_get("is_starred")?,
        ))
    }

    fn map_row_to_feed_log(row: &Row) -> RResult<FeedLog> {
        Ok(FeedLog {
            feed_id: row.try_get("feed_id")?,
            last_pub_date: row.try_get("last_pub_date")?,
            healthy: row.try_get("healthy")?,
            log: row.try_get("log")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
        })
    }

    fn map_row_to_feed(row: &Row) -> RResult<Feed> {
        let feed_type: String = row.try_get("feed_type")?;
        let feed_type = FeedTypeMark::from_str(&feed_type)?;

        Ok(Feed {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            link: row.try_get("link")?,
            description: row.try_get("description")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
            parent_id: row.try_get("parent_id")?,
            item_type: feed_type,
            logo: row.try_get("logo")?,
            feed_url: row.try_get("feed_url")?,
            sort: row.try_get("sort")?,
            sync_interval_sec: row.try_get("sync_interval_sec")?,
        })
    }
}

#[async_trait]
impl Mapper for PostgresMapper {
    async fn global_search(&self, search: SearchReq) -> RResult<Vec<SearchArtileRsp>> {
        let stmt = self.pool.get().await?;

        let query = format!("%{}%", search.query);
        let limit = search.limit.unwrap_or(12) as i64;
        let cursor = ((search.cursor.as_ref().unwrap_or(&1) - 1) as i64 * limit) as i64;

        let result = stmt
            .query(
                "
        SELECT
          A.*
        FROM
          feed f
        LEFT JOIN article a
        on a.feed_id = f.id
        WHERE
          (
            A.title ILIKE $1
            OR
            A.content ILIKE $2
            OR
            A.description ILIKE $2
          )
          AND A.feed_id = F.id
        LIMIT $3 OFFSET $4",
                &[&query.as_str(), &query.as_str(), &limit, &cursor],
            )
            .await?
            .iter()
            .map(|e| SearchArtileRsp {
                article: Self::map_row_to_article(e).unwrap(),
            })
            .collect();

        Ok(result)
    }

    async fn get_web_cache(&self, url: &str) -> RResult<WebCache> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .query("select * from web_caches where url = $1", &[&url])
            .await?;

        if row.is_empty() {
            anyhow::bail!("unable fetch")
        } else {
            let row = row.get(0).ok_or(anyhow::anyhow!("unable to get"))?;
            let cache = WebCache {
                id: row.get("id"),
                url: row.get("url"),
                content_type: row.get("content_type"),
            };

            Ok(cache)
        }
    }

    async fn set_web_cache(&self, cache: &WebCache) -> RResult<u64> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .execute(
                "insert into web_caches (id, url, content_type) values ($1,$2,$3)",
                &[&cache.id, &cache.url, &cache.content_type],
            )
            .await?;

        Ok(row)
    }
}

macro_rules! cond_with {
    ($sql:expr, $num:expr) => {
        if $num == 0 {
            $sql.push_str(" where ");
            $num = 1;
        } else if $num == 1 {
            $num = 2;
        } else if $num == 2 {
            $sql.push_str(" and ");
        }
    };
}

#[async_trait]
impl ArticleMapper for PostgresMapper {
    async fn get_article_with_uuid(&self, uuid: &str) -> RResult<Article> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .query_one("select * from article where id = $1", &[&uuid])
            .await?;

        Ok(Self::map_row_to_article(&row)?)
    }

    async fn get_article_with_url(&self, url: &str) -> RResult<Article> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .query_one("select * from article where link = $1", &[&url])
            .await?;

        Ok(Self::map_row_to_article(&row)?)
    }

    async fn set_article_content_with_url(&self, url: &str, content: &str) -> RResult<()> {
        let stmt = self.pool.get().await?;
        stmt.execute(
            "update article set cached_content = $1 where link = $2",
            &[&content, &url],
        )
        .await?;

        Ok(())
    }

    async fn update_article_read_status(&self, uuid: &str, status: bool) -> RResult<usize> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .execute(
                "update article set is_read = $1 where id = $2",
                &[&status, &uuid],
            )
            .await?;

        Ok(row as usize)
    }

    async fn update_article_star_status(&self, uuid: &str, status: bool) -> RResult<usize> {
        let stmt = self.pool.get().await?;

        let row = stmt
            .execute(
                "update article set is_starred = $1 where id = $2",
                &[&status, &uuid],
            )
            .await?;

        Ok(row as usize)
    }

    async fn get_articles(&self, filter: &ArticleFilterReq) -> RResult<Vec<Article>> {
        let stmt = self.pool.get().await?;
        let mut sql = String::new();
        let mut with_and = 0;

        sql.push_str("select a.* from feed f left join article a on a.feed_id = f.id");

        if let Some(id) = &filter.feed_id {
            cond_with!(sql, with_and);

            let ids_sql = self.get_sub_ids_as_sql_seg(id).await?;

            sql.push_str("f.id in ");
            sql.push_str(ids_sql.as_str());
            sql.push_str(" ");

            cond_with!(sql, with_and);
        }

        if let Some(true) = filter.is_today {
            cond_with!(sql, with_and);

            sql.push_str(" DATE(a.pub_time) = DATE('now') ");
        }

        if let Some(r) = filter.is_read {
            cond_with!(sql, with_and);

            match r {
                true => sql.push_str(" is_read = true "),
                false => sql.push_str(" is_read = false "),
            }
        }

        if let Some(true) = filter.is_starred {
            cond_with!(sql, with_and);

            sql.push_str(" is_starred = true ");
        }

        sql.push_str(" order by is_starred desc, pub_time desc ");

        if let Some(limit) = filter.limit {
            sql.push_str(format!(" limit {} ", limit).as_str());

            if let Some(offset) = filter.cursor {
                sql.push_str(
                    format!(" offset {} ", ((offset - 1) as i64 * limit as i64) as i64).as_str(),
                );
            }
        }

        info!("sql is: {}", sql);

        let rows = stmt
            .query(sql.as_str(), &[])
            .await?
            .iter()
            .filter_map(|e| match Self::map_row_to_article(e) {
                Ok(o) => {
                    return Some(o);
                }
                Err(e) => {
                    info!("unable to map {}", e);
                    return None;
                }
            })
            .collect();

        Ok(rows)
    }

    async fn add_articles(&self, articles: &Vec<Article>) -> RResult<u64> {
        let stmt = self.pool.get().await?;
        let mut count = 0;

        for a in articles {
            let c = stmt.execute("insert into article(id, pub_id, title, link, feed_id, description, author, pub_time, content, create_time, update_time, is_read, is_starred,feed_title) 
            values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) on conflict (id) do nothing",
             &[&a.get_id(), &a.pub_id, &a.title, &a.link, &a.feed_id, &a.description, &a.author, &a.pub_time, &a.content, &a.create_time, &a.update_time, &a.is_read, &a.is_starred, &a.feed_title]).await?;
            count += c;
        }

        Ok(count)
    }
}

#[async_trait]
impl FeedMapper for PostgresMapper {
    async fn update_folder_name(&self, uuid: &str, name: &str) -> RResult<()> {
        let stmt = self.pool.get().await?;

        stmt.execute("update feed set title = $1 where id = $2", &[&name, &uuid])
            .await?;

        Ok(())
    }

    async fn get_sub_ids(&self, id: &str) -> RResult<Vec<String>> {
        let stmt = self.pool.get().await?;

        let mut ids: HashSet<String> = HashSet::new();

        stmt.query(
            "with recursive children(id, parent_id) as (
select n.id, n.parent_id from feed n where n.parent_id = $1
union 
select n.id, n.parent_id from feed n, children c where n.parent_id = c.id
)
select * from children;",
            &[&id],
        )
        .await?
        .iter()
        .for_each(|e| {
            ids.insert(e.get("id"));
            ids.insert(e.get("parent_id"));
        });

        ids.insert(id.to_owned());

        Ok(ids.into_iter().collect())
    }

    async fn move_channel_into_folder(&self, sub_id: &str, parent_id: &str) -> RResult<()> {
        let stmt = self.pool.get().await?;
        stmt.execute(
            format!("update feed set parent_id = $1 where id = $2").as_str(),
            &[&parent_id, &sub_id],
        )
        .await?;
        Ok(())
    }

    async fn get_sub_true_feeds(&self, id: &str) -> RResult<Vec<Feed>> {
        let seg = self.get_sub_ids_as_sql_seg(id).await?;

        let stmt = self.pool.get().await?;

        Ok(stmt
            .query(
                format!("select * from feed where id in {} and feed_type = $1", seg).as_str(),
                &[&FeedTypeMark::Feed.as_ref()],
            )
            .await?
            .iter()
            .filter_map(|e| Self::map_row_to_feed(e).ok())
            .collect())
    }

    async fn write_log(&self, log: &FeedLog) -> RResult<()> {
        let stmt = self.pool.get().await?;

        stmt.execute("insert into feed_log(feed_id, last_pub_date, healthy, log, create_time, update_time) values($1,$2,$3,$4,$5,$6)", &[&log.feed_id, &log.last_pub_date, &log.healthy, &log.log, &log.create_time, &log.update_time]).await?;

        Ok(())
    }

    async fn mark_as_read(&self, id: &MarkAllReadReq) -> RResult<()> {
        let ids: Vec<Feed> = match id.is_all {
            Some(true) => self.get_all_feeds().await?,
            _ => match id.uuid.as_ref() {
                Some(id) => self.get_sub_true_feeds(&id).await?,
                None => {
                    vec![]
                }
            },
        };

        for feed in ids {
            let stmt = self.pool.get().await?;
            stmt.execute(
                "update article set is_read = true where feed_id = $1",
                &[&feed.id],
            )
            .await?;
        }

        Ok(())
    }

    async fn get_feed_logs(&self, id: &str) -> RResult<Vec<FeedLog>> {
        let stmt = self.pool.get().await?;

        let res = stmt
            .query(
                "select * from feed_log where feed_id = $1 order by create_time desc",
                &[&id],
            )
            .await?
            .iter()
            .filter_map(|e| Some(Self::map_row_to_feed_log(e).unwrap()))
            .collect();

        Ok(res)
    }

    async fn get_last_success_feed_log(&self, id: &str) -> Option<FeedLog> {
        let logs = self.get_feed_logs(id).await.ok()?;

        logs.into_iter()
            .filter(|e| e.healthy)
            .max_by(|x, y| x.create_time.cmp(&y.create_time))
    }

    async fn update_feed_sync_interval(&self, id: &str, interval: u32) -> RResult<u64> {
        let stmt = self.pool.get().await?;

        Ok(stmt
            .execute(
                "update feed set sync_interval_sec = $1 where id = $2",
                &[&(interval as i32), &id],
            )
            .await?)
    }

    async fn get_all_feeds(&self) -> RResult<Vec<Feed>> {
        let stmt = self.pool.get().await?;

        Ok(stmt
            .query("select * from feed", &[])
            .await?
            .iter()
            .filter_map(|e| {
                let f = Self::map_row_to_feed(e);
                match f {
                    Ok(feed) => Some(feed),
                    Err(err) => {
                        error!("unable parse this to lines: {}", err);
                        None
                    }
                }
            })
            .collect())
    }

    async fn delete_feed(&self, id: &str) -> RResult<u64> {
        let stmt = self.pool.get().await?;

        Ok(stmt
            .execute("delete from feed where id = $1", &[&id])
            .await?)
    }

    async fn add_feed(&self, feed: &Feed) -> RResult<()> {
        let stmt = self.pool.get().await?;

        stmt.execute("insert into feed(id,parent_id,title,link,feed_type,logo,feed_url,description,create_time,update_time,sort) values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)", 
         &[&feed.id,&feed.parent_id,&feed.title,&feed.link,&feed.item_type.as_ref(),&feed.logo,&feed.feed_url,&feed.description,&feed.create_time,&feed.update_time,&feed.sort]
        ).await?;

        Ok(())
    }

    async fn update_feed_sort(&self, sorts: &Vec<FeedSortReq>) -> RResult<u64> {
        let stmt = self.pool.get().await?;

        for sort in sorts {
            let parent = if sort.folder_uuid == sort.uuid {
                ""
            } else {
                sort.folder_uuid.as_str()
            };
            stmt.execute(
                "update feed set parent_id = $1, sort = $2 where id = $3",
                &[&parent, &sort.sort, &sort.uuid],
            )
            .await?;
        }

        Ok(sorts.len() as u64)
    }

    async fn count_per_feed(&self, req: &FeedsCountReq) -> RResult<HashMap<String, i64>> {
        let stmt = self.pool.get().await?;
        let mut map = HashMap::new();
        let sql = format!(
            "SELECT
            feed_id,
            count(is_read) as unread_count
          FROM article
          WHERE {}
          GROUP BY feed_id",
            match req.filter {
                0 => " is_starred = true ",
                1 => " is_read = false ",
                _ => " 1 = 1 ",
            }
        );
        info!("sql: {}", sql);

        stmt.query(sql.as_str(), &[]).await?.iter().for_each(|row| {
            let key: String = row.get("feed_id");
            let count: i64 = row.get("unread_count");
            map.insert(key, count);
        });

        Ok(map)
    }

    async fn get_collection_metas(&self) -> RResult<CollectionMeta> {
        let stmt = self.pool.get().await?;

        let res = stmt
            .query_one(
                "
      SELECT
        COUNT(1) AS today,
        (SELECT COUNT(1) FROM article WHERE is_read = false) AS total
      FROM article
      WHERE DATE(create_time) = DATE('now') AND is_read = true",
                &[],
            )
            .await?;

        Ok(CollectionMeta {
            total: res.get("total"),
            today: res.get("today"),
        })
    }
}
