use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use async_trait::async_trait;
use chin_tools::wrapper::anyhow::AResult;
use clap::builder::Str;
use deadpool_postgres::{
    tokio_postgres::{self, Row},
    Pool,
};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::{
    mapper::feed_rs::ring_id,
    model::{
        db::{Article, BlockedLink, Feed, FeedLog, FeedTypeMark, WebCache},
        dto::{
            ArticleFilterReq, CollectionMeta, FeedSortReq, FeedsCountReq, MarkAllReadReq,
            SearchArticleRsp, SearchReq,
        },
    },
    tool::htmlconverter::contains_only_text,
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

    fn map_row_to_article(row: &Row) -> AResult<Article> {
        let ring_id = row.try_get("ring_id")?;
        Ok(Article::new_from_db(
            row.try_get("id")?,
            ring_id,
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

    fn map_row_to_feed_log(row: &Row) -> AResult<FeedLog> {
        Ok(FeedLog {
            feed_id: row.try_get("feed_id")?,
            last_pub_date: row.try_get("last_pub_date")?,
            healthy: row.try_get("healthy")?,
            log: row.try_get("log")?,
            create_time: row.try_get("create_time")?,
            update_time: row.try_get("update_time")?,
        })
    }

    fn map_row_to_feed(row: &Row) -> AResult<Feed> {
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
    async fn fix_db(&self) -> AResult<()> {
        warn!("fix db");
        let stmt = self.pool.get().await?;

        let full: i64 = 500;
        let mut count: i64 = 501;
        let mut start: i64 = 0;

        while full <= count {
            let res = stmt.query(
                "select a.id, a.pub_id, f.feed_url from article a join feed f on a.feed_id = f.id order by a.create_time asc limit $1 offset $2",
                &[&full, &start],
            ).await?;

            count = res.len() as i64;
            start += count;
            info!("update {}", start);

            for ele in res.iter() {
                let pub_id: Option<String> = ele.get("pub_id");
                let id: String = ele.get("id");
                stmt.execute(
                    "update article set ring_id = $1 where id = $2",
                    &[&ring_id(pub_id.as_ref(), ele.get("feed_url")), &id],
                )
                .await?;
            }
        }

        warn!("fix db done");

        Ok(())
    }

    async fn global_search(&self, search: SearchReq) -> AResult<Vec<SearchArticleRsp>> {
        let stmt = self.pool.get().await?;

        let query = format!("%{}%", search.query);
        let limit = search.limit.unwrap_or(12) as i64;
        let cursor = ((search.cursor.as_ref().unwrap_or(&1) - 1) as i64 * limit) as i64;
        let sql = "
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
            A.description ILIKE $2
          )
          AND A.feed_id = F.id
        order by create_time desc
        LIMIT $3 OFFSET $4";

        info!("{} /// {}", sql, query);
        let result = stmt
            .query(sql, &[&query.as_str(), &query.as_str(), &limit, &cursor])
            .await?
            .iter()
            .map(|e| SearchArticleRsp {
                article: Self::map_row_to_article(e).unwrap(),
            })
            .collect();

        Ok(result)
    }

    async fn get_web_cache(&self, url: &str) -> AResult<WebCache> {
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

    async fn set_web_cache(&self, cache: &WebCache) -> AResult<u64> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .execute(
                "insert into web_caches (id, url, content_type) values ($1,$2,$3)",
                &[&cache.id, &cache.url, &cache.content_type],
            )
            .await?;

        Ok(row)
    }

    async fn add_blocked_domain(&self, url: &str) -> AResult<u64> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .execute("insert into blocked_domains (url) values ($1) ", &[&url])
            .await?;

        Ok(row)
    }

    async fn fetch_blocked_domains(&self) -> AResult<Vec<BlockedLink>> {
        let stmt = self.pool.get().await?;
        let rows = stmt.query("select * from blocked_domains", &[]).await?;

        let ls = rows
            .iter()
            .map(|e| BlockedLink {
                url: e.get("url"),
                insert_time: e.get("insert_time"),
                update_time: e.get("update_time"),
            })
            .collect();

        Ok(ls)
    }
}

macro_rules! cond_with {
    ($sql:expr, $num:expr) => {
        if $num == 0 {
            $sql.push_str(" where ");
            $num = 1;
        } else if $num == 1 {
            $sql.push_str(" and ");
        }
    };
}

#[async_trait]
impl ArticleMapper for PostgresMapper {
    async fn get_article_with_uuid(&self, uuid: &str) -> AResult<Article> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .query_one("select * from article where id = $1", &[&uuid])
            .await?;

        Ok(Self::map_row_to_article(&row)?)
    }

    async fn get_article_with_url(&self, url: &str) -> AResult<Article> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .query_one("select * from article where link = $1 limit 1", &[&url])
            .await?;

        Ok(Self::map_row_to_article(&row)?)
    }

    async fn set_article_content_with_url(&self, url: &str, content: &str) -> AResult<()> {
        let stmt = self.pool.get().await?;
        stmt.execute(
            "update article set cached_content = $1 where link = $2",
            &[&content, &url],
        )
        .await?;

        Ok(())
    }

    async fn update_article_read_status(&self, uuid: &str, status: bool) -> AResult<usize> {
        let stmt = self.pool.get().await?;
        let row = stmt
            .execute(
                "update article set is_read = $1 where id = $2",
                &[&status, &uuid],
            )
            .await?;

        Ok(row as usize)
    }

    async fn update_article_star_status(&self, uuid: &str, status: bool) -> AResult<usize> {
        let stmt = self.pool.get().await?;

        let row = stmt
            .execute(
                "update article set is_starred = $1 where id = $2",
                &[&status, &uuid],
            )
            .await?;

        Ok(row as usize)
    }

    async fn get_articles(&self, filter: &ArticleFilterReq) -> AResult<Vec<Article>> {
        let stmt = self.pool.get().await?;
        let mut sql = String::new();
        let mut with_cond = 0;

        sql.push_str("select a.* from feed f left join article a on a.feed_id = f.id ");

        if let Some(id) = &filter.feed_id {
            cond_with!(sql, with_cond);

            let ids_sql = self.get_sub_ids_as_sql_seg(id).await?;

            sql.push_str("f.id in ");
            sql.push_str(ids_sql.as_str());
            sql.push_str(" ");
        }

        if let Some(true) = filter.is_today {
            cond_with!(sql, with_cond);

            sql.push_str(" DATE(a.pub_time) = DATE('now') ");
        }

        if let Some(r) = filter.is_read {
            cond_with!(sql, with_cond);

            match r {
                true => sql.push_str(" is_read = true "),
                false => sql.push_str(" is_read = false "),
            }
        }

        if let Some(true) = filter.is_starred {
            cond_with!(sql, with_cond);

            sql.push_str(" is_starred = true ");
        }

        cond_with!(sql, with_cond);
        sql.push_str(" is_current = true ");

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

    async fn add_articles(&self, articles: &Vec<Article>) -> AResult<u64> {
        let stmt = self.pool.get().await?;
        let mut count = 0;

        for a in articles {
            let inserted = stmt
                .query(
                    "select title, description, content from article where ring_id = $1",
                    &[&a.ring_id],
                )
                .await?;

            let do_insert = !inserted.iter().any(|r| {
                let title: String = r.try_get("title").map_or("".to_string(), |e| e);
                let description: String = r.try_get("description").map_or("".to_string(), |e| e);
                let content: String = r.try_get("content").map_or("".to_string(), |e| e);

                contains_only_text(title.as_str(), a.title.trim())
                    && contains_only_text(description.as_str(), a.description.trim())
                    && contains_only_text(content.as_str(), a.content.trim())
            });

            if !inserted.is_empty() && do_insert {
                stmt.execute(
                    "update article set is_current = false where ring_id = $1",
                    &[&a.ring_id],
                )
                .await?;
            }

            if do_insert {
                let c = stmt.execute("insert into article(id, ring_id, pub_id, title, link, feed_id, description, author, pub_time, content, create_time, update_time, is_read, is_starred,feed_title) 
            values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
             &[&a.get_id(), &a.ring_id, &a.pub_id, &a.title, &a.link, &a.feed_id, &a.description, &a.author, &a.pub_time, &a.content, &a.create_time, &a.update_time, &a.is_read, &a.is_starred, &a.feed_title]).await?;

                count += c;
            }
        }

        Ok(count)
    }
}

#[async_trait]
impl FeedMapper for PostgresMapper {
    async fn update_folder_name(&self, uuid: &str, name: &str) -> AResult<()> {
        let stmt = self.pool.get().await?;

        stmt.execute("update feed set title = $1 where id = $2", &[&name, &uuid])
            .await?;

        Ok(())
    }

    async fn get_sub_ids(&self, id: &str) -> AResult<Vec<String>> {
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

    async fn move_channel_into_folder(&self, sub_id: &str, parent_id: &str) -> AResult<()> {
        let stmt = self.pool.get().await?;
        stmt.execute(
            format!("update feed set parent_id = $1 where id = $2").as_str(),
            &[&parent_id, &sub_id],
        )
        .await?;
        Ok(())
    }

    async fn get_sub_true_feeds(&self, id: &str) -> AResult<Vec<Feed>> {
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

    async fn write_log(&self, log: &FeedLog) -> AResult<()> {
        let stmt = self.pool.get().await?;

        stmt.execute("insert into feed_log(feed_id, last_pub_date, healthy, log, create_time, update_time) values($1,$2,$3,$4,$5,$6)", &[&log.feed_id, &log.last_pub_date, &log.healthy, &log.log, &log.create_time, &log.update_time]).await?;

        Ok(())
    }

    async fn mark_as_read(&self, id: &MarkAllReadReq) -> AResult<()> {
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

    async fn get_feed_logs(&self, id: &str) -> AResult<Vec<FeedLog>> {
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

    async fn update_feed_sync_interval(&self, id: &str, interval: u32) -> AResult<u64> {
        let stmt = self.pool.get().await?;

        Ok(stmt
            .execute(
                "update feed set sync_interval_sec = $1 where id = $2",
                &[&(interval as i32), &id],
            )
            .await?)
    }

    async fn get_all_feeds(&self) -> AResult<Vec<Feed>> {
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

    async fn delete_feed(&self, id: &str) -> AResult<u64> {
        let stmt = self.pool.get().await?;

        Ok(stmt
            .execute("delete from feed where id = $1", &[&id])
            .await?)
    }

    async fn add_feed(&self, feed: &Feed) -> AResult<()> {
        let stmt = self.pool.get().await?;

        stmt.execute("insert into feed(id,parent_id,title,link,feed_type,logo,feed_url,description,create_time,update_time,sort) values($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)", 
         &[&feed.id,&feed.parent_id,&feed.title,&feed.link,&feed.item_type.as_ref(),&feed.logo,&feed.feed_url,&feed.description,&feed.create_time,&feed.update_time,&feed.sort]
        ).await?;

        Ok(())
    }

    async fn update_feed_sort(&self, sorts: &Vec<FeedSortReq>) -> AResult<u64> {
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

    async fn count_per_feed(&self, req: &FeedsCountReq) -> AResult<HashMap<String, i64>> {
        let stmt = self.pool.get().await?;
        let mut map = HashMap::new();
        let sql = format!(
            "SELECT
            feed_id,
            count(is_read) as unread_count
          FROM article
          WHERE {} and is_current = true
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

        stmt.query("select id, parent_id from feed", &[])
            .await?
            .iter()
            .for_each(|row| {
                let parent: Option<String> = row.get("parent_id");
                let id: String = row.get("id");

                let s = map.get(&id).unwrap_or(&0);
                if let Some(parent) = parent {
                    let p = map.get(&parent).unwrap_or(&0);

                    map.insert(parent, p + s);
                }
            });

        Ok(map)
    }

    async fn get_collection_metas(&self) -> AResult<CollectionMeta> {
        let stmt = self.pool.get().await?;

        let res = stmt
            .query_one(
                "
      SELECT
        COUNT(1) AS today,
        (SELECT COUNT(1) FROM article WHERE is_read = false) AS total
      FROM article
      WHERE DATE(create_time) = DATE('now') AND is_read = true and is_current = true",
                &[],
            )
            .await?;

        Ok(CollectionMeta {
            total: res.get("total"),
            today: res.get("today"),
        })
    }
}
