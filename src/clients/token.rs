use anyhow::Result;
use log::{debug, warn};
use reqwest::Client;
use serde_json::{json, Value};
use std::{
    env,
    time::{Duration, Instant},
};

const SUCCESS_CODE: u64 = 200;
const SYSTEM_ERROR_CODE: u16 = 500;

#[derive(Debug)]
pub struct TokenClient {
    pub client: Client,
}

impl TokenClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()?;
        Ok(Self { client })
    }

    // Get value from api.
    pub async fn get_value(&self, key: &str) -> Result<String> {
        let url = env::var("TOKEN_SERVER_URL").unwrap();

        let params = json!({
            "key": key,
        });

        let data = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await?
            .json::<Value>()
            .await?;

        let code = data
            .get("code")
            .unwrap_or(&SYSTEM_ERROR_CODE.into())
            .as_u64()
            .unwrap();

        Ok(match code {
            SUCCESS_CODE => {
                let value = data["data"]["value"].as_str().unwrap().to_string();
                debug!("Get {}:{}", key, value);
                value
            }
            _ => {
                warn!("Fail to get {}.", key);
                "".to_string()
            }
        })
    }

    // Get bx ua.
    pub async fn get_bx_ua(&self) -> Result<String> {
        let start = Instant::now();
        let bx_ua = self.get_value("bx_ua").await?;
        debug!("获取bx_ua: {:?}, 花费时间:{:?}", bx_ua, start.elapsed());
        Ok(bx_ua)
    }

    // Get bx token.
    pub async fn get_bx_token(&self) -> Result<String> {
        let start = Instant::now();
        let bx_token = self.get_value("bx_token").await?;
        debug!(
            "获取bx_token: {:?}, 花费时间:{:?}",
            bx_token,
            start.elapsed()
        );
        Ok(bx_token)
    }
}
