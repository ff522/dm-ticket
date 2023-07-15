use std::env;

use crate::clients::token::TokenClient;
use crate::models::qrcode::{
    QrCodeLoginGetResForm, QrCodeLoginGetResParams, QrCodeLoginStatusData, QrcodeContentGetParams,
    QrcodeData,
};
use crate::models::DmLoginRes;
use anyhow::Result;
use fast_qr::{QRBuilder, QRCode};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, Value};
use tokio::{fs, io::AsyncWriteExt};
use urlencoding;

#[derive(Debug)]
pub struct LoginClient {
    pub token_client: TokenClient,
    pub client: Client,
}

impl LoginClient {
    pub async fn new() -> Result<Self> {
        let redis_url = env::var("REDIS_URL").unwrap();
        let token_client = TokenClient::new(redis_url).await?;

        let mut headers = HeaderMap::new();
        headers.append("user-agent", HeaderValue::from_str("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")?);
        headers.append("referer", HeaderValue::from_str("https://ipassport.damai.cn/mini_login.htm?lang=zh_cn&appName=damai&appEntrance=default&styleType=vertical&bizParams=&notLoadSsoView=true&notKeepLogin=false&isMobile=false&showSnsLogin=false&regUrl=https%3A%2F%2Fpassport.damai.cn%2Fregister&plainReturnUrl=https%3A%2F%2Fpassport.damai.cn%2Flogin&returnUrl=https%3A%2F%2Fpassport.damai.cn%2Fdologin.htm%3FredirectUrl%3Dhttps%25253A%25252F%25252Fwww.damai.cn%25252F%26platform%3D106002&rnd=0.6260742856882737")?);
        headers.append(
            "content-type",
            HeaderValue::from_str("application/json;charset=UTF-8")?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .http2_prior_knowledge()
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 13_2_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/13.0.3")
            .use_rustls_tls()
            .build()?;
        Ok(Self {
            token_client,
            client,
        })
    }

    pub async fn request(&self, url: &str, params: Value, mut data: Value) -> Result<DmLoginRes> {
        data["bx-umidtoken"] = self.token_client.get_bx_token().await?.into();
        data["bx-ua"] = self.token_client.get_bx_ua().await?.into();

        let response = self
            .client
            .post(url)
            .query(&params)
            .form(&data)
            .send()
            .await?;

        let data = response.json::<DmLoginRes>().await?;

        Ok(data)
    }

    // 生成二维码
    pub async fn generate_qrcode(&self) -> Result<QrcodeData> {
        let url = "https://ipassport.damai.cn/newlogin/qrcode/generate.do";
        let res = self
            .request(url, QrcodeContentGetParams::build()?, json!({}))
            .await?;
        let data = serde_json::from_value(res.content.data)?;
        Ok(data)
    }

    // 获取二维码
    pub async fn get_qrcode(&self, qrcode_content: String) -> Result<QRCode> {
        let qrcode_path = env::var("QRCODE_PATH").unwrap();
        let url = format!(
            "https://gcodex.alicdn.com/qrcode.do?biz_code=havana&size=140&content={}",
            urlencoding::encode(&qrcode_content)
        );
        let mut source = self.client.get(&url).send().await?;

        let mut dest = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&qrcode_path)
            .await?;

        while let Some(chunk) = source.chunk().await? {
            dest.write_all(&chunk).await?;
        }

        let img = image::open(&qrcode_path)?.to_luma8();

        let mut img = rqrr::PreparedImage::prepare(img);

        let grids = img.detect_grids();
        let (_, content) = grids[0].decode()?;

        let qrcode = QRBuilder::new(content).build().unwrap();

        let _ = fs::remove_file(qrcode_path).await;

        Ok(qrcode)
    }

    // 获取登录结果
    pub async fn get_login_result(&self, t: u64, ck: String) -> Result<QrCodeLoginStatusData> {
        let url = "https://ipassport.damai.cn/newlogin/qrcode/query.do";
        let mut params = QrCodeLoginGetResParams::build()?;
        params["ua"] = self.token_client.get_ua().await?.into();
        let form_data = QrCodeLoginGetResForm::build(t, ck)?;
        let res = self.request(url, params, form_data).await?;
        let data = serde_json::from_value(res.content.data)?;
        Ok(data)
    }

    // 获取cookie
    pub async fn get_cookie(
        &self,
        cookie2: &String,
        return_url: String,
        st: String,
    ) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.append("user-agent", HeaderValue::from_str("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")?);
        headers.append(
            "referer",
            HeaderValue::from_str(
                "https://passport.damai.cn/login?ru=https%3A%2F%2Fwww.damai.cn%2F",
            )?,
        );
        headers.append(
            "cookie",
            HeaderValue::from_str(&format!("cookie2={};", cookie2))?,
        );
        let url = format!("{}&st={}", return_url, st);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .http2_prior_knowledge()
            .use_rustls_tls()
            .build()?;

        let _ = client.get(url).send().await?;

        let response = client.get("https://passport.damai.cn/accountinfo/myinfo?spm=a2oeg.home.nick.duserinfo.591b23e1MY5Dhp").send().await?;

        let mut cookies = format!("cookie2={};", cookie2);
        for (name, value) in response.headers() {
            let name = name.to_string();
            let value = value.to_str().unwrap().to_string();
            if name.starts_with("set-cookie") {
                let values = value.split(' ').collect::<Vec<&str>>();
                let cookie = values[0];
                cookies.push_str(cookie);
            }
        }

        Ok(cookies.to_string())
    }
}
