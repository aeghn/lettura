use serde::{Deserialize, Serialize};

pub mod alias;
pub mod constant;
pub mod custom;
pub mod db;
pub mod dto;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub ip: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
}
