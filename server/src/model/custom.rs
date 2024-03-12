use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use super::ProxyConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomizeStyle {
    typeface: String,
    font_size: i32,
    line_height: f32,
    line_width: i32,
}

impl Default for CustomizeStyle {
    fn default() -> Self {
        Self {
            typeface: String::from("var(--sans-font)"),
            font_size: 14,
            line_height: 1.4,
            line_width: 600,
        }
    }
}

macro_rules! generate_set_property {
    ($config:ident, $method:ident, $field:ident, $field_type:ty) => {
        pub fn $method(mut $config, value: $field_type) -> Self {
            $config.$field = value;
            $config
        }
    };
    ($config:ident, $method:ident, $field:ident, Option<$field_type:ty>) => {
        pub fn $method(mut $config, value: Option<$field_type>) -> Self {
            $config.$field = value;
            $config
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub threads: i32,
    pub theme: String,

    pub update_interval: u64,
    pub last_sync_time: String,

    pub local_proxy: Option<ProxyConfig>,
    pub customize_style: CustomizeStyle,
    pub purge_on_days: u64,
    pub purge_unread_articles: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            threads: 1,
            theme: String::from('1'),
            update_interval: 0,
            last_sync_time: Utc
                .timestamp_millis_opt(0)
                .unwrap()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            local_proxy: None,
            customize_style: CustomizeStyle::default(),
            purge_on_days: 0,
            purge_unread_articles: true,
        }
    }
}
