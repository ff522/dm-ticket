use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::CommonParams;

pub struct GetUserInfoParams;

impl GetUserInfoParams {
    pub fn build() -> Result<Value> {
        let mut params = serde_json::to_value(CommonParams::build())?;
        params["api"] = "mtop.damai.wireless.user.session.transform".into();
        params["v"] = "1.0".into();
        Ok(params)
    }
}

pub struct GetUserInfoForm;

impl GetUserInfoForm {
    pub fn build() -> Result<Value> {
        Ok(json!({"source":"h5","dmChannel":"damai@damaih5_h5"}))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfoData {
    pub nickname: String,

    #[serde(rename = "userId")]
    pub user_id: u64,
}
