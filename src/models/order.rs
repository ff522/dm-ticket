use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::CommonParams;

// 大麦生成订单接口params
pub struct OrderParams;
impl OrderParams {
    pub fn build() -> Result<Value> {
        let mut params = serde_json::to_value(CommonParams::build())?;
        params["v"] = "4.0".into();
        params["api"] = "mtop.trade.order.build.h5".into();
        params["method"] = "POST".into();
        params["ttid"] = "#t#ip##_h5_2014".into();
        params["globalCode"] = "ali.china.damai".into();
        params["tb_eagleeyex_scm_project"] = "20190509-aone2-join-test".into();
        params["AntiFlood"] = "true".into();
        Ok(params)
    }
}

pub struct OrderForm;

// 生成订单表单参数
impl OrderForm {
    pub fn build(item_id: &String, sku_id: &String, by_num: usize) -> Result<Value> {
        let ext_params = json!({
            "channel": "damai_app",
            "damai": "1",
            "umpChannel": "100031004",
            "subChannel": "damai@damaih5_h5",
            "atomSplit": "1",
            "serviceVersion": "2.0.0",
            "customerType": "default"
        });

        let data = json!({
            "buyNow": "true",
            "exParams": serde_json::to_string(&ext_params)?,
            "buyParam": format!("{}_{}_{}", item_id, by_num, sku_id),
            "dmChannel": "damai@damaih5_h5"
        });
        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoContainer;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoGlobal {
    #[serde(rename = "secretKey")]
    pub secret_key: String,

    #[serde(rename = "secretValue")]
    pub secret_value: String, // submitref
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoHierarchy {
    pub component: Vec<String>,
    pub root: String,

    #[serde(rename = "baseType")]
    pub base_type: Vec<String>,

    pub structure: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoLinkageCommon {
    #[serde(rename = "queryParams")]
    pub query_params: String,

    pub compress: bool,

    #[serde(rename = "validateParams")]
    pub validate_params: String,

    pub structures: String,

    #[serde(rename = "submitParams")]
    pub submit_params: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfoLinkage {
    pub input: Vec<String>,
    pub request: Vec<String>,
    pub signature: String,
    pub common: OrderInfoLinkageCommon,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderInfo {
    // pub container: OrderInfoContainer,
    pub data: Value,
    //pub endpoint: OrderInfoEndpoint,
    pub global: OrderInfoGlobal,
    pub hierarchy: OrderInfoHierarchy,
    pub linkage: OrderInfoLinkage,
}

// 提交订单params
pub struct SubmitOrderParams;
impl SubmitOrderParams {
    pub fn build(submitref: String) -> Result<Value> {
        let mut params = serde_json::to_value(CommonParams::build())?;
        params["api"] = "mtop.trade.order.create.h5".into();
        params["v"] = "4.0".into();
        params["submitref"] = submitref.into();
        params["timeout"] = "15000".into();
        params["isSec"] = "1".into();
        params["ecode"] = "1".into();
        params["post"] = "1".into();
        params["ttid"] = "#t#ip##_h5_2014".into();
        params["globalCode"] = "ali.china.damai".into();
        params["tb_eagleeyex_scm_project"] = "20190509-aone2-join-test".into();
        Ok(params)
    }
}
