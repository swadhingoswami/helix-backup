use log::LevelFilter;

pub fn init_logger(level: Option<&str>) {
    let log_level = match level.unwrap_or("info") {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_millis()
        .format_target(true)
        .format_module_path(true)
        .init();
}

pub fn init_file_logger(path: &str, level: Option<&str>) -> Result<(), anyhow::Error> {
    let log_level = match level.unwrap_or("info") {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let file = std::fs::File::create(path)?;

    env_logger::Builder::new()
        .filter_level(log_level)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .format_timestamp_millis()
        .init();

    Ok(())
}
