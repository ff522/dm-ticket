use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::CommonParams;

// 查询门票信息表单
#[derive(Serialize, Deserialize, Debug)]
pub struct TicketInfoForm {
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "dmChannel")]
    dm_channel: &'static str,
}

impl TicketInfoForm {
    pub fn build(ticket_id: String) -> Result<Value> {
        let data = Self {
            item_id: ticket_id,
            dm_channel: "damai@damaih5_h5",
        };
        Ok(serde_json::to_value(data)?)
    }
}

// 查询门票信息参数
#[derive(Serialize, Deserialize, Debug)]
pub struct TicketInfoParams {}
impl TicketInfoParams {
    pub fn build() -> Result<Value> {
        let mut params = serde_json::to_value(CommonParams::build())?;
        params["api"] = "mtop.alibaba.damai.detail.getdetail".into();
        params["v"] = "1.2".into();
        Ok(params)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sku {
    #[serde(rename = "skuId")]
    pub sku_id: String,

    #[serde(rename = "skuName")]
    pub sku_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Perform {
    #[serde(rename = "performId")]
    pub perform_id: String, // 演出ID

    #[serde(rename = "itemId")]
    pub item_id: String, // 场次ID

    #[serde(rename = "performName")]
    pub perform_name: String, //演出名称

                              // #[serde(rename = "skuList")]
                              // pub sku_list: Vec<Sku>, // sku 列表
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerformBase {
    pub name: String,

    #[serde(rename = "timeSpan")]
    pub time_span: String,

    #[serde(rename = "performBaseTagDesc")]
    pub perform_base_tag_desc: String,

    // 场次。取索引0
    pub performs: Vec<Perform>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketDetail {
    #[serde(rename = "sellStartTime")]
    pub sell_start_timestamp: String,

    #[serde(rename = "sellStartTimeStr")]
    pub sell_start_time_str: String,

    #[serde(rename = "performBases")]
    pub perform_bases: Vec<PerformBase>, // 演出场次列表, 账号设置选择索引
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticDataItemBase {
    #[serde(rename = "itemId")]
    pub item_id: String,

    #[serde(rename = "itemName")]
    pub item_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StaticData {
    #[serde(rename = "itemBase")]
    pub item_base: StaticDataItemBase,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetailViewComponentItem {
    #[serde(rename = "staticData")]
    pub static_data: StaticData,

    #[serde(rename = "dynamicExtData")]
    pub dynamic_ext_data: Value,

    pub item: TicketDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetailViewComponentMap {
    pub atmosphere: Value,
    pub item: DetailViewComponentItem,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketInfo {
    #[serde(rename = "detailViewComponentMap")]
    pub detail_view_component_map: DetailViewComponentMap,
}
