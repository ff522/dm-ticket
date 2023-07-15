use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub nickname: String,
    pub ticket_id: String,               // 门票ID
    pub ticket_name: String,             // 门票名称
    pub ticket_perform_id: String,       // 门票场次ID
    pub ticket_perform_name: String,     // 门票场次名称
    pub ticket_perform_sku_id: String,   // 门票票档ID
    pub ticket_perform_sku_name: String, // 门票票档名称
    pub ticket_num: usize,               // 购票数量
    pub priority_purchase_time: i64,     // 优先购时长
    pub request_time_offset: i64,        // 请求时间偏移量
    pub retry_interval: u64,             // 重试间隔
    pub retry_times: u64,                // 重试次数
    pub wait_for_submit_interval: u64,   // 生成/提交订单的间隔

    // 实名人选择
    #[serde(default = "default_real_names")]
    pub real_names: Vec<usize>,
}

// 实名人, 默认自动选择前ticket->num位。
fn default_real_names() -> Vec<usize> {
    vec![]
}
