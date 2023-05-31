use anyhow::Result;
use dm_ticket::{
    config::{load_global_config, Config},
    dm,
};
use dotenv::dotenv;
use futures::future::join_all;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("TOKEN_SERVER_URL").is_err() {
        env::set_var("TOKEN_SERVER_URL", "http://127.0.0.1:8080/");
    }

    pretty_env_logger::init();

    let config: Config = load_global_config().unwrap();

    let mut handlers = Vec::new();

    for account in config.accounts.iter() {
        let account = account.clone();
        let handler = tokio::spawn(async move {
            let dm_ticket = dm::DmTicket::new(account).await.unwrap();
            dm_ticket.run().await.unwrap();
        });
        handlers.push(handler);
    }
    join_all(handlers).await;

    Ok(())
}
