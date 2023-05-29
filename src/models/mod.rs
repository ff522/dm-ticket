pub mod order;
pub mod perform;
pub mod qrcode;
pub mod ticket;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::{value, Value};
// cookie token.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DmToken {
    pub token_with_time: String,
    pub token: String,
    pub enc_token: String,
}

// 大麦API返回数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DmRes {
    pub api: Option<String>,
    pub data: value::Value,
    pub ret: Vec<String>,
    pub v: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DmLoginResContent {
    pub status: i32,
    pub success: bool,
    pub data: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DmLoginRes {
    #[serde(rename = "hasError")]
    pub has_error: bool,

    pub content: DmLoginResContent,
}

// 通用参数
#[derive(Serialize, Deserialize, Debug)]
pub struct CommonParams {
    jsv: &'static str,

    #[serde(rename = "appKey")]
    app_key: &'static str,

    r#type: &'static str,

    #[serde(rename = "dataType")]
    data_type: &'static str,

    #[serde(rename = "H5Request")]
    h5_request: &'static str,

    #[serde(rename = "AntiCreep")]
    anti_creep: &'static str,

    #[serde(rename = "AntiFlood")]
    anti_flood: &'static str,

    t: String,

    #[serde(rename = "requestStart")]
    request_start: String,
}

impl CommonParams {
    pub fn build() -> Self {
        let local: DateTime<Local> = Local::now();
        let millis = local.timestamp_millis();

        Self {
            jsv: "2.7.2",
            app_key: "12574478",
            r#type: "originaljson",
            data_type: "json",
            h5_request: "true",
            anti_creep: "true",
            anti_flood: "true",
            t: millis.to_string(),
            request_start: (millis - 1).to_string(),
        }
    }
}

impl Default for CommonParams {
    fn default() -> Self {
        Self::build()
    }
}
