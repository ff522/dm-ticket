use std::env;

use anyhow::Result;
use dm_ticket::login::DmLogin;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }

    if env::var("TOKEN_SERVER_URL").is_err() {
        env::set_var("TOKEN_SERVER_URL", "http://127.0.0.1:8080/");
    }

    if env::var("QRCODE_PATH").is_err() {
        env::set_var("QRCODE_PATH", "./qrcode.png");
    }

    pretty_env_logger::init();

    let app = DmLogin::new().await?;
    app.run().await?;

    Ok(())
}
