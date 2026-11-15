use crate::config::{Config, rpc::RpcConfig};
use color_eyre::{Result, eyre::eyre};
use std::{fs::create_dir_all, path::PathBuf};
use tracing::{Level, level_filters::LevelFilter};
use tracing_appender::rolling;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    EnvFilter, filter::filter_fn, layer::SubscriberExt, util::SubscriberInitExt,
};

const TRANSMISSION_RPC_TARGET: &str = "transmission_rpc";

/// Installs the global JSON file logger with levels from `config.log`.
///
/// # Errors
///
/// Returns an error if the local data directory cannot be determined or the log
/// directory cannot be created.
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
    let rpc_cap = transmission_rpc_cap(&config.rpc);

    tracing_subscriber::registry()
        .with(JsonStorageLayer)
        .with(formatter)
        .with(EnvFilter::new(format!(
            "traxor={},ratatui={},transmission_rpc={}",
            config.log.traxor, config.log.ratatui, config.log.transmission_rpc,
        )))
        .with(filter_fn(move |metadata| {
            is_allowed(metadata.target(), *metadata.level(), rpc_cap)
        }))
        .init();

    Ok(())
}

/// Most verbose level allowed for `transmission_rpc` events, whatever `[log]` says.
///
/// transmission-rpc 0.5 logs its `BasicAuth`, plaintext password included, at
/// `debug` before every request (`TransClient::call`), and the `tracing-log`
/// bridge forwards it to the log file. Cap the crate at `info` while
/// credentials are configured.
const fn transmission_rpc_cap(rpc: &RpcConfig) -> LevelFilter {
    if rpc.username.is_some() || rpc.password.is_some() {
        LevelFilter::INFO
    } else {
        LevelFilter::TRACE
    }
}

fn is_allowed(target: &str, level: Level, rpc_cap: LevelFilter) -> bool {
    let is_transmission_rpc = target
        .strip_prefix(TRANSMISSION_RPC_TARGET)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"));
    !is_transmission_rpc || level <= rpc_cap
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rpc(username: Option<&str>, password: Option<&str>) -> RpcConfig {
        RpcConfig {
            username: username.map(Into::into),
            password: password.map(Into::into),
            ..RpcConfig::default()
        }
    }

    #[test]
    fn caps_transmission_rpc_only_with_credentials() {
        assert_eq!(transmission_rpc_cap(&rpc(None, None)), LevelFilter::TRACE);
        assert_eq!(
            transmission_rpc_cap(&rpc(Some("user"), Some("secret"))),
            LevelFilter::INFO
        );
        assert_eq!(
            transmission_rpc_cap(&rpc(Some("user"), None)),
            LevelFilter::INFO
        );
    }

    #[test]
    fn blocks_transmission_rpc_debug_under_info_cap() {
        let cap = LevelFilter::INFO;
        assert!(!is_allowed("transmission_rpc", Level::DEBUG, cap));
        assert!(!is_allowed("transmission_rpc::types", Level::TRACE, cap));
        assert!(is_allowed("transmission_rpc", Level::INFO, cap));
        assert!(is_allowed("transmission_rpc", Level::ERROR, cap));
    }

    #[test]
    fn leaves_other_targets_alone() {
        let cap = LevelFilter::INFO;
        assert!(is_allowed("traxor", Level::TRACE, cap));
        assert!(is_allowed("transmission_rpc_other", Level::DEBUG, cap));
        assert!(is_allowed(
            "transmission_rpc",
            Level::DEBUG,
            LevelFilter::TRACE
        ));
    }
}
