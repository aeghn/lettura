use std::sync::Arc;

use crate::mapper::db::{
    postgres::{PostgresConfig, PostgresMapper},
    Mapper,
};
use anyhow::Ok;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum DbConfig {
    #[serde(rename = "postgres")]
    Postgres(PostgresConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Common {
    pub asset_base_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Proxy {
    pub ip: String,
    pub port: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackupConfig {
    pub dir: String,
    pub interval: Option<u32>,
}

impl DbConfig {
    pub fn into(self) -> anyhow::Result<Arc<(dyn Mapper + 'static)>> {
        let mapper = match self {
            DbConfig::Postgres(pg) => {
                let mapper = PostgresMapper::new(pg)?;
                Arc::new(mapper) as Arc<dyn Mapper>
            }
            /*             DbConfig::Sqlite(cfg) => Arc::new(SqliteMapper::new(cfg)?) as Arc<dyn Mapper>,
             */
            _ => todo!(),
        };

        Ok(mapper)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "mapper")]
    pub db_config: DbConfig,
    pub common: Common,
    pub backup: Option<BackupConfig>,
    pub proxy: Option<Proxy>,
}
