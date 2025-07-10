use crate::config::Config;
use color_eyre::{Result, eyre::eyre};
use std::{fs::create_dir_all, path::PathBuf};
use tracing_appender::rolling;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logger(config: &Config) -> Result<()> {
    let log_dir_path = if cfg!(debug_assertions) {
        PathBuf::from(".logs")
    } else {
        let mut path =
            dirs::data_local_dir().ok_or_else(|| eyre!("Failed to get local data directory"))?;
        path.push("traxor/logs");
        path
    };
    create_dir_all(&log_dir_path)?;

    let logfile = if cfg!(debug_assertions) {
        rolling::daily(log_dir_path, "traxor.log")
    } else {
        rolling::never(log_dir_path, "traxor.log")
    };

    let formatter = BunyanFormattingLayer::new("traxor".into(), logfile);

    tracing_subscriber::registry()
        .with(JsonStorageLayer)
        .with(formatter)
        .with(EnvFilter::new(format!(
            "traxor={},ratatui={},transmission_rpc={}",
            config.log.traxor, config.log.ratatui, config.log.transmission_rpc,
        )))
        .init();

    Ok(())
}
