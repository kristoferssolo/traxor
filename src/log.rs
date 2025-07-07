use color_eyre::eyre::Result;
use tracing_appender::rolling;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup_logger() -> Result<()> {
    std::fs::create_dir_all(".logs")?;
    let logfile = rolling::daily(".log", "traxor.log");
    let log_layer = tracing_subscriber::fmt::layer()
        .with_writer(logfile)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(log_layer)
        .with(EnvFilter::from_default_env())
        .init();

    Ok(())
}
