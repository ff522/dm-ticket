use anyhow::Result;
use dm_ticket::client::Client;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("WEBDRIVER_URL").is_err() {
        env::set_var("WEBDRIVER_URL", "http://localhost:9515");
    }

    if env::var("REDIS_URL").is_err() {
        env::set_var("REDIS_URL", "redis://127.0.0.1:6379/0");
    }

    if env::var("QRCODE_PATH").is_err() {
        env::set_var("QRCODE_PATH", ".qrcode.png");
    }

    pretty_env_logger::init();

    let webdriver_url = env::var("WEBDRIVER_URL").unwrap();
    let client = Client::new(webdriver_url).await?;

    client.run().await?;
    Ok(())
}
