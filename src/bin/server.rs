use anyhow::Result;
use dm_ticket::server::Server;
use dotenv::dotenv;
use log::error;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("REDIS_URL").is_err() {
        env::set_var("REDIS_URL", "redis://127.0.0.1:6379/0");
    }

    if env::var("WEBDRIVER_URL").is_err() {
        env::set_var("WEBDRIVER_URL", "http://localhost:9515");
    }

    if env::var("TOTAL_TOKEN_NUM").is_err() {
        env::set_var("TOTAL_TOKEN_NUM", "100");
    }

    if env::var("BATCH_TOKEN_NUM").is_err() {
        env::set_var("BATCH_TOKEN_NUM", "10");
    }

    pretty_env_logger::init();

    let webdriver_url = env::var("WEBDRIVER_URL").unwrap();
    let redis_url = env::var("REDIS_URL").unwrap();

    match Server::new(webdriver_url, redis_url).await {
        Ok(mut server) => {
            match server.serve().await {
                Ok(_) => {}
                Err(e) => {
                    error!("服务启动失败, 原因:{}!", e)
                }
            };
        }
        Err(e) => {
            error!("服务启动失败, 原因:{}!", e.to_string());
        }
    }
    Ok(())
}
