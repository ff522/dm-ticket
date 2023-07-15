use super::token::TokenClient;
use crate::models::{ticket::TicketInfoParams, DmRes, DmToken};
use anyhow::Result;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, Value};

#[derive(Debug)]
pub struct DmClient {
    pub client: Client,
    pub token_client: Option<TokenClient>,
    pub token: DmToken,
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
    pub async fn new(cookie: Option<String>, token_client: Option<TokenClient>) -> Result<Self> {
        let cookie = cookie
            .unwrap_or("".to_string())
            .replace(' ', "")
            .replace('\n', "")
            .split(';')
            .filter(|e| !e.starts_with("_m_h5_tk"))
            .collect::<Vec<&str>>()
            .join(";");

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
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")
            .use_rustls_tls()
            .build()?;
        Ok(Self {
            client,
            token,
            token_client,
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

        if self.token_client.is_some() {
            let token_client = self.token_client.clone().unwrap();
            params["bx-umidtoken"] = token_client.get_bx_token().await?.into();
            params["bx-ua"] = token_client.get_bx_ua().await?.into();
        }

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
