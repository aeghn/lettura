use arguments::Arguments;
use clap::Parser;
use config::Config;
use model::alias::RResult;
use tracing::{info, Level};

pub mod arguments;
pub mod config;
pub mod controller;
pub mod mapper;
pub mod model;
pub mod tool;

#[tokio::main]
async fn main() -> RResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_thread_ids(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let args = Arguments::parse();

    let config_file = tokio::fs::read_to_string(args.config.as_str()).await;
    match config_file {
        Ok(cf) => {
            let config: Config = toml::from_str(cf.as_str())?;
            controller::serve(config).await;
        }
        Err(err) => {
            info!("unable to read err, creating default config to it. {}", err);
        }
    }

    Ok(())
}
