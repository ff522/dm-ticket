use anyhow::Result;
use log::info;
use redis::{AsyncCommands, Client};
use std::{env, time::Duration};
use thirtyfour::{
    prelude::ElementQueryable, By, ChromeCapabilities, DesiredCapabilities, WebDriver,
};
use tokio::signal;

use crate::errors::ServerError;

const KEY_BX_UA: &str = "bx_ua";

const KEY_UMID_TOKEN: &str = "bx_umid_token";

const KEY_UA: &str = "ua";

pub struct Server {
    webdriver_url: String,
    driver: WebDriver,
    client: Client,
    caps: ChromeCapabilities,
}

impl Server {
    pub async fn new(webdriver_url: String, redis_url: String) -> Result<Self> {
        let mut caps = DesiredCapabilities::chrome();

        caps.set_disable_dev_shm_usage()?;
        caps.set_headless()?;
        caps.set_disable_gpu()?;
        caps.set_disable_web_security()?;
        caps.set_ignore_certificate_errors()?;
        caps.add_chrome_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_chrome_arg("--disable-logging")?;
        caps.add_chrome_arg("--blink-settings=imagesEnabled=false")?;
        caps.add_chrome_arg("--incognito")?;
        caps.add_chrome_arg("--disable-stylesheet")?;
        caps.add_chrome_arg("--excludeSwitches=[\"enable-automation\"]")?;
        caps.add_chrome_arg("--useAutomationExtension=false")?;
        caps.add_chrome_arg("--disable-infobars")?;
        caps.add_chrome_arg("--disable-software-rasterizer")?;
        caps.add_chrome_arg("--disable-extensions")?;
        caps.add_chrome_arg("--no-sandbox")?;
        caps.add_chrome_arg("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36")?;
        caps.add_chrome_arg("--window-size=1920,1080")?;
        caps.add_chrome_arg("--single-process")?;

        let client = redis::Client::open(redis_url.clone())
            .map_err(|_| ServerError::RedisConnectionError)?;

        let driver: WebDriver = WebDriver::new(&webdriver_url, caps.clone())
            .await
            .map_err(|_| ServerError::WebdriverConnectionError)?;

        let _ = driver.clone().quit().await;
        Ok(Server {
            webdriver_url,
            client,
            driver,
            caps,
        })
    }

    pub async fn refresh_driver(&mut self) -> Result<()> {
        self.driver = WebDriver::new(&self.webdriver_url, self.caps.clone())
            .await
            .map_err(|_| ServerError::WebdriverConnectionError)?;

        let url = "https://login.m.taobao.com/login.htm?redirectURL=https%3A%2F%2Ferror.taobao.com%2Fapp%2Ftbhome%2Fcommon%2Ferror.html&loginFrom=wap_tbTop";

        self.driver.goto(url).await?;

        let _ = self
            .driver
            .query(By::Css("div[class='body']"))
            .wait(Duration::from_secs(10), Duration::from_millis(100));

        Ok(())
    }

    pub(crate) async fn exec_js(&self, js: &str) -> Result<String> {
        let value = self.driver.execute(js, vec![]).await?;

        let value = value.json().to_string().replace('"', "");

        Ok(value)
    }

    pub(crate) async fn get_bx_ua(&self) -> Result<String> {
        let js = "return window.__baxia__.postFYModule.getFYToken();";

        let bx_ua = self.exec_js(js).await?;

        Ok(bx_ua)
    }

    pub(crate) async fn get_bx_umid_token(&self) -> Result<String> {
        let js = "return window.__baxia__.postFYModule.getUidToken();";

        let bx_umid_token = self.exec_js(js).await?;

        Ok(bx_umid_token)
    }

    pub(crate) async fn get_ua(&self) -> Result<String> {
        let js = "return window.__nc.__uab.getUA();";

        let bx_umid_token = self.exec_js(js).await?;

        Ok(bx_umid_token)
    }

    pub(crate) async fn get_token_num(&self) -> Result<u32> {
        let mut conn = self.client.get_async_connection().await?;

        let cnt = conn.llen(KEY_BX_UA).await?;

        Ok(cnt)
    }

    pub(crate) async fn insert_bx_ua(&self, value: String) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;

        conn.lpush(KEY_BX_UA, value).await?;

        Ok(())
    }

    pub(crate) async fn insert_bx_umid_token(&self, value: String) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;

        conn.lpush(KEY_UMID_TOKEN, value).await?;

        Ok(())
    }

    pub(crate) async fn insert_ua(&self, value: String) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;

        conn.lpush(KEY_UA, value).await?;

        Ok(())
    }

    pub async fn quit(&self) -> Result<()> {
        self.driver.clone().quit().await?;

        Ok(())
    }

    pub async fn serve(&mut self) -> Result<()> {
        let total_token_num = env::var("TOTAL_TOKEN_NUM")
            .unwrap()
            .parse::<u32>()
            .map_err(|_| ServerError::InvalidConfig {
                key: "TOTAL_TOKEN_NUM".into(),
            })?;

        let batch_token_num = env::var("BATCH_TOKEN_NUM")
            .unwrap()
            .parse::<u32>()
            .map_err(|_| ServerError::InvalidConfig {
                key: "MAX_TOKEN_NUM".into(),
            })?;

        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {

                    let cur_token_num = self.get_token_num().await?;

                    if cur_token_num > total_token_num {
                        info!("当前token数量:{}, 已大于配置的TOTAL_TOKEN_NUM:{}, 故不再获取!", cur_token_num, total_token_num);
                        continue;
                    }

                    info!("正在刷新客户端...");
                    self.refresh_driver().await?;

                    info!("正在获取token...");
                    for _ in 0..batch_token_num {
                        if let Ok(bx_umid_token) = self.get_bx_umid_token().await{
                            let _ = self.insert_bx_umid_token(bx_umid_token).await;
                        }
                        if let Ok(bx_ua) = self.get_bx_ua().await {
                            let _ = self.insert_bx_ua(bx_ua).await;
                        }
                        if let Ok(ua) = self.get_ua().await {
                            let _ = self.insert_ua(ua).await;
                        }
                    }
                    info!("成功获取{}个token...", batch_token_num);
                }

                _ = signal::ctrl_c() => {
                    info!("CTRL-C退出程序...");
                    self.quit().await?;
                    return Ok(());
                }
            }
        }
    }
}
