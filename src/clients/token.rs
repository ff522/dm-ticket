use anyhow::Result;
use redis::{AsyncCommands, Client};

use crate::errors::ClientError;

#[derive(Debug, Clone)]
pub struct TokenClient {
    client: Client,
}

impl TokenClient {
    pub async fn new(redis_url: String) -> Result<Self> {
        let client =
            redis::Client::open(redis_url).map_err(|_| ClientError::RedisConnectionError)?;

        Ok(Self { client })
    }

    // 获取大麦请求头bx-ua
    pub async fn get_bx_ua(&self) -> Result<String> {
        let mut con = self.client.get_async_connection().await?;
        let bx_ua: String = con.lpop("bx_ua", None).await?;
        Ok(bx_ua)
    }

    pub async fn get_ua(&self) -> Result<String> {
        let mut con = self.client.get_async_connection().await?;
        let bx_ua: String = con.lpop("ua", None).await?;
        Ok(bx_ua)
    }

    // 获取大麦请求头bx-umidtoken
    pub async fn get_bx_token(&self) -> Result<String> {
        let mut con = self.client.get_async_connection().await?;
        let bx_token: String = con.lpop("bx_umid_token", None).await?;
        Ok(bx_token)
    }
}
