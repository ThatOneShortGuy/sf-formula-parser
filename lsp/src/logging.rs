use std::{fs, io, path::Path};

use anyhow::{Context, Result};
use tracing_subscriber::{
    Layer, filter, fmt::time::ChronoLocal, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn setup_logging_to_stderr_and_file(
    file_path: impl AsRef<Path>,
    // stderr_log_level: filter::LevelFilter,
) -> Result<()> {
    let stderr_log_level = filter::LevelFilter::DEBUG;
    let file_log_level = filter::LevelFilter::INFO;
    let stderr_layer = tracing_subscriber::fmt::layer().with_writer(io::stderr);

    let file_layer = tracing_subscriber::fmt::layer().with_writer(
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path.as_ref())
            .context("opening logging file")?,
    );

    tracing_subscriber::registry()
        .with(
            stderr_layer
                .with_timer(ChronoLocal::rfc_3339())
                .with_filter(stderr_log_level),
        )
        .with(
            file_layer
                .with_timer(ChronoLocal::rfc_3339())
                .with_ansi(false)
                .with_filter(file_log_level),
        )
        .try_init()?;

    Ok(())
}
