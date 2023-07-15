use anyhow::Result;
use serde_json::{json, Value};

pub struct NotifyClient {}

impl NotifyClient {
    pub async fn notify(content: &str) -> Result<bool> {
        let token = std::env::var("NOTIFY_TOKEN").unwrap();
        let client = reqwest::Client::builder().cookie_store(true).build()?;

        let url = format!("http://www.pushplus.plus/send/{}", token);

        let params = json!({
            "title": "抢票结果通知",
            "content": content,
            "channel": "wechat",
            "template": "markdown",
        });

        let response = client
            .post(url)
            .query(&params)
            .send()
            .await?
            .json::<Value>()
            .await?;

        if response
            .get("code")
            .unwrap_or(&Value::from(500))
            .as_i64()
            .unwrap_or(500)
            == 200
        {
            return Ok(true);
        }
        Ok(false)
    }
}
