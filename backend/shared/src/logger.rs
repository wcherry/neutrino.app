use tracing_appender::{rolling, non_blocking};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt, fmt::time::UtcTime};

pub fn init_logging(log_level: &str, log_path: Option<String>) {
    let stdout_layer = fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .with_writer(std::io::stdout);

    let sub = tracing_subscriber::registry()
        .with(EnvFilter::new(log_level))
        .with(stdout_layer);

    if let Some(path) = log_path {
        let file_appender = rolling::daily(path, "service.log");
        let (file_writer, _guard) = non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_timer(UtcTime::rfc_3339())
            .with_writer(file_writer);
        sub.with(file_layer).init();
    } else {
        sub.init();
    }
}