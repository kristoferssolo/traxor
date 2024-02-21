use std::{fs::File, path::PathBuf, str::FromStr};

use tracing::Level;
use tracing_subscriber::fmt;

pub fn setup_logger() {
    std::fs::create_dir_all(".logs").unwrap();
    let path = PathBuf::from_str(".logs/traxor.log").unwrap();
    let log_file = File::create(path).expect("Failed to create log file");
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(log_file)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
