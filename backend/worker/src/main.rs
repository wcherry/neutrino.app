use log::info;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    std::env::set_var("RUST_LOG", &log_level);
    env_logger::init();

    info!("Worker starting...");

    // Future background task processing will be implemented here.
    // Examples: feed refresh, scraping, email sending, analytics aggregation.

    info!("Worker exiting cleanly.");
}
