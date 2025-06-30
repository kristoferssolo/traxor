use std::{fs::File, path::PathBuf, str::FromStr};

use anyhow::Result;
use tracing::Level;
use tracing_subscriber::fmt;

pub fn setup_logger() -> Result<()> {
    std::fs::create_dir_all(".logs")?;
    let path = PathBuf::from_str(".logs/traxor.log")?;
    let log_file = File::create(path)?;
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(log_file)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
