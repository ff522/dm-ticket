use crate::models::{ticket::TicketInfoParams, DmRes, DmToken};
use anyhow::Result;

use super::token::TokenClient;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, Value};

#[derive(Debug)]
pub struct DmClient {
    pub client: Client,
    pub token_client: TokenClient,
    pub token: DmToken,
    pub bx_token: String,
}

// 获取token
pub async fn get_token(cookie: &str) -> Result<DmToken> {
    let mut headers = HeaderMap::new();
    let url = "https://mtop.damai.cn/";

    headers.append("origin", HeaderValue::from_str(url)?);
    headers.append("referer", HeaderValue::from_str(url)?);
    headers.append("cookie", HeaderValue::from_str(cookie)?);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .http2_prior_knowledge()
        .build()?;

    let mut token = DmToken {
        enc_token: "".to_string(),
        token_with_time: "".to_string(),
        token: "".to_string(),
    };

    let url = "https://mtop.damai.cn/h5/mtop.damai.wireless.search.broadcast.list/1.0/?";
    let params = TicketInfoParams::build()?;
    let response = client.get(url).form(&params).send().await?;

    for cookie in response.cookies() {
        if cookie.name() == "_m_h5_tk" {
            token.token_with_time = cookie.value().to_string();
            token.token = token.token_with_time.split('_').collect::<Vec<_>>()[0].to_string();
        }
        if cookie.name() == "_m_h5_tk_enc" {
            token.enc_token = cookie.value().to_string();
        }
    }
    Ok(token)
}

impl DmClient {
    // 初始化请求客户端
    pub async fn new(cookie: String) -> Result<Self> {
        let token_client = TokenClient::new()?;

        let bx_token = token_client.get_bx_token().await?;

        let token = get_token(&cookie).await?;

        let mut headers = HeaderMap::new();

        let base_url = "https://mtop.damai.cn/";
        headers.append("origin", HeaderValue::from_str(base_url)?);
        headers.append("referer", HeaderValue::from_str(base_url)?);

        headers.append(
            "cookie",
            HeaderValue::from_str(
                format!(
                    "{};_m_h5_tk_enc={};_m_h5_tk={};",
                    &cookie, token.enc_token, token.token_with_time
                )
                .as_str(),
            )?,
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .http2_prior_knowledge()
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3")
            .use_rustls_tls()
            .build()?;
        Ok(Self {
            client,
            token,
            token_client,
            bx_token,
        })
    }

    // 请求API
    pub async fn request(&self, url: &str, mut params: Value, data: Value) -> Result<DmRes> {
        let s = format!(
            "{}&{}&{}&{}",
            self.token.token,
            params["t"].as_str().unwrap(),
            params["appKey"].as_str().unwrap(),
            serde_json::to_string(&data)?,
        );

        let sign = format!("{:?}", md5::compute(s));

        params["sign"] = sign.into();

        params["bx-umidtoken"] = self.bx_token.clone().into();
        params["bx-ua"] = self.token_client.get_bx_ua().await?.into();

        let form = json!({
            "data": serde_json::to_string(&data)?,
        });

        let response = self
            .client
            .post(url)
            .query(&params)
            .form(&form)
            .send()
            .await?;

        let data = response.json::<DmRes>().await?;

        Ok(data)
    }
}
