use std::{
    env,
    io::{self, Write},
    time::{Duration, Instant},
};

use crate::rand_i64;

use crate::{
    clients::{dm::DmClient, token::TokenClient},
    models::{
        order::{OrderForm, OrderInfo, OrderParams, SubmitOrderParams},
        task::Task,
        ticket::{TicketInfo, TicketInfoForm, TicketInfoParams},
        user::{GetUserInfoForm, GetUserInfoParams, UserInfoData},
        DmRes,
    },
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, TimeZone};
use log::{debug, error, info, warn};
use serde_json::json;
use tokio::signal;

const SUCCESS_FLAG: &str = "SUCCESS::调用成功";

pub struct DmTicket {
    pub client: DmClient,
    pub task: Task,
}

impl DmTicket {
    // Construct
    pub async fn new(cookie: String, task: Task) -> Result<Self> {
        let redis_url = env::var("REDIS_URL").unwrap();
        let token_client = TokenClient::new(redis_url).await?;

        let client = DmClient::new(Some(cookie), Some(token_client)).await?;

        Ok(Self { client, task })
    }

    // 获取用户信息
    pub async fn get_user_info(&self) -> Result<UserInfoData> {
        let url = "https://mtop.damai.cn/h5/mtop.damai.wireless.user.session.transform/1.0/";
        let params = GetUserInfoParams::build()?;
        let form = GetUserInfoForm::build()?;
        let res = self.client.request(url, params, form).await?;
        if res.ret.contains(&SUCCESS_FLAG.to_string()) {
            let user_info_data = serde_json::from_value(res.data)?;
            Ok(user_info_data)
        } else {
            Err(anyhow!("{}", res.ret[0]))
        }
    }

    // 获取门票信息
    pub async fn get_ticket_info(&self, ticket_id: String) -> Result<TicketInfo> {
        let url = "https://mtop.damai.cn/h5/mtop.alibaba.damai.detail.getdetail/1.2";

        let params = TicketInfoParams::build()?;

        let data = TicketInfoForm::build(&ticket_id)?;

        let res = self.client.request(url, params, data).await?;

        match res.ret.contains(&SUCCESS_FLAG.to_string()) {
            true => {
                debug!("{}, 获取门票信息成功, {:?}", self.task.nickname, res);

                let ticket_info: TicketInfo =
                    serde_json::from_str(res.data["result"].clone().as_str().unwrap())?;
                Ok(ticket_info)
            }
            false => {
                error!(
                    "{}, 获取门票信息失败, 结果:{:?}",
                    self.task.nickname, res.ret
                );
                Err(anyhow!("获取门票信息失败..."))
            }
        }
    }

    // 生成订单
    pub async fn build_order(
        &self,
        item_id: &String,
        sku_id: &String,
        buy_num: usize,
    ) -> Result<OrderInfo> {
        let start = Instant::now();

        let url = "https://mtop.damai.cn/h5/mtop.trade.order.build.h5/4.0/?";

        let params = OrderParams::build()?;

        let data = OrderForm::build(item_id, sku_id, buy_num)?;

        let res = self.client.request(url, params, data).await?;

        debug!(
            "{}, 生成订单结果:{:?}, 花费时间:{:?}",
            self.task.nickname,
            res,
            start.elapsed()
        );

        match res.ret.contains(&SUCCESS_FLAG.to_string()) {
            true => {
                let order_info: OrderInfo = serde_json::from_value(res.data)?;
                Ok(order_info)
            }
            false => Err(anyhow!("{:?}", res.ret)),
        }
    }

    // 提交订单
    pub async fn submit_order(&self, order_info: OrderInfo) -> Result<DmRes> {
        let start = Instant::now();

        let url = "https://mtop.damai.cn/h5/mtop.trade.order.create.h5/4.0/";

        // 添加提交订单需要的数据
        let mut order_data = json!({});

        for key in order_info.linkage.input.iter() {
            if key.starts_with("dmViewer_") {
                let mut item = order_info.data[key].clone();
                let mut num = self.task.ticket_num;

                let viewer_list = item["fields"]["viewerList"].clone();

                // 需选择实名观演人
                if viewer_list.is_array() && !viewer_list.as_array().unwrap().is_empty() {
                    // 实名观演人比购票数量少
                    if viewer_list.as_array().unwrap().len() < num {
                        warn!("实名观演人小于实际购票数量, 请先添加实名观演人!");
                        num = viewer_list.as_array().unwrap().len();
                    }
                    if self.task.real_names.is_empty() {
                        info!(
                            "{}, 未配置实名观演人, 默认选择前{}位观演人...",
                            self.task.nickname, self.task.ticket_num
                        );
                        for i in 0..num {
                            item["fields"]["viewerList"][i]["isUsed"] = true.into();
                        }
                    } else {
                        for i in 0..item["fields"]["viewerList"]
                            .as_array()
                            .unwrap_or(&Vec::new())
                            .len()
                        {
                            let idx = i + 1;
                            if self.task.real_names.contains(&idx) {
                                item["fields"]["viewerList"][i]["isUsed"] = true.into();
                            }
                        }
                    }
                }
                order_data[key] = item;
            } else {
                order_data[key] = order_info.data[key].clone();
            }
        }

        let confirm_order_key = &order_info.hierarchy.root;
        order_data[confirm_order_key] = order_info.data[confirm_order_key].clone();

        let keys_list = order_info.hierarchy.structure[confirm_order_key].clone();

        for k in keys_list.as_array().unwrap() {
            let s = k.as_str().unwrap();
            if s.starts_with("order_") {
                order_data[s] = order_info.data[s].clone();
            }
        }

        let order_hierarchy = json!({
            "structure": order_info.hierarchy.structure
        });

        let order_linkage = json!({
            "common": {
                "compress": order_info.linkage.common.compress,
                "submitParams": order_info.linkage.common.submit_params,
                "validateParams": order_info.linkage.common.validate_params,
            },
            "signature": order_info.linkage.signature,
        });

        let submit_order_params = SubmitOrderParams::build(order_info.global.secret_value)?;

        let feature = json!({
            "subChannel": "damai@damaih5_h5",
            "returnUrl": "https://m.damai.cn/damai/pay-success/index.html?spm=a2o71.orderconfirm.bottom.dconfirm&sqm=dianying.h5.unknown.value",
            "serviceVersion": "2.0.0",
            "dataTags": "sqm:dianying.h5.unknown.value"
        });

        let params = json!({
            "data": serde_json::to_string(&order_data)?,
            "hierarchy": serde_json::to_string(&order_hierarchy)?,
            "linkage": serde_json::to_string(&order_linkage)?,
        });

        let sumbit_order_data = json!({
            "params": serde_json::to_string(&params)?,
            "feature": serde_json::to_string(&feature)?,
        });

        let res = self
            .client
            .request(url, submit_order_params, sumbit_order_data)
            .await?;
        debug!("提交订单结果:{:?}, 花费时间:{:?}", res, start.elapsed());

        Ok(res)
    }

    // 毫秒转时分秒
    pub fn ms_to_hms(&self, ms: i64) -> (u64, u64, f64) {
        let sec = ms as f64 / 1000.0;
        let hour = (sec / 3600.0) as u64;
        let rem = sec % 3600.0;
        let min = (rem / 60.0) as u64;
        let sec = rem % 60.0;
        (hour, min, sec)
    }

    // 尝试多次购买
    pub async fn multiple_buy_attempts(
        &self,
        item_id: &String,
        sku_id: &String,
        buy_num: Option<usize>,
    ) -> Result<bool> {
        let buy_num = match buy_num {
            Some(num) => num,
            None => self.task.ticket_num,
        };
        let mut retry_times = self.task.retry_times;
        if retry_times < 1 {
            retry_times = 3;
        }

        let mut order_info: Option<OrderInfo> = None;

        for i in 0..retry_times {
            let start = Instant::now();
            order_info = match self.build_order(item_id, sku_id, buy_num).await {
                Ok(data) => {
                    info!(
                        "\n{}, 第{}次生成订单成功, 耗时:{:?}毫秒\n",
                        Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                        i + 1,
                        start.elapsed().as_millis()
                    );
                    Some(data)
                }
                Err(e) => {
                    error!(
                        "\n{}, 第{}次生成订单失败, 耗时:{:?}毫秒, {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                        i + 1,
                        start.elapsed().as_millis(),
                        e.to_string()
                    );

                    let retry_interval = rand_i64(self.task.retry_interval as i64);
                    tokio::time::sleep(Duration::from_millis(retry_interval)).await;
                    continue;
                }
            };
            break;
        }

        if order_info.is_none() {
            return Err(anyhow!("生成订单失败!"));
        }

        let wait_for_submit_time = rand_i64(self.task.wait_for_submit_interval as i64);

        tokio::time::sleep(Duration::from_millis(wait_for_submit_time)).await;

        for _ in 0..retry_times {
            let start = Instant::now();
            let order = order_info.clone();
            let res = self.submit_order(order.unwrap()).await?;
            match res.ret.contains(&SUCCESS_FLAG.to_string()) {
                true => {
                    info!(
                        "{}, {}, 提交订单成功, 请尽快前往手机APP付款,  耗时:{}毫秒!",
                        Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                        self.task.nickname,
                        start.elapsed().as_millis()
                    );
                    return Ok(true);
                }
                false => {
                    info!(
                        "{}, {}, 提交订单失败, 原因:{}, 耗时:{:?}毫秒",
                        Local::now().format("%Y-%m-%d %H:%M:%S.%3f"),
                        self.task.nickname,
                        res.ret[0],
                        start.elapsed().as_millis()
                    );
                    let retry_interval = rand_i64(self.task.retry_interval as i64);
                    tokio::time::sleep(Duration::from_millis(retry_interval)).await;
                }
            };
        }
        Ok(false)
    }

    // 程序入口
    pub async fn run(&mut self) -> Result<()> {
        info!("{}, 正在检查用户信息...", self.task.nickname);
        let user_info = match self.get_user_info().await {
            Ok(info) => info,
            Err(e) => {
                if e.to_string().contains("FAIL_SYS_SESSION_EXPIRED::Session") {
                    error!(
                        "{}, 获取用户信息失败, cookie已过期, 请重新登陆!",
                        self.task.nickname,
                    );
                } else {
                    error!("{}, 获取用户信息失败, 原因:{:?}", self.task.nickname, e);
                }
                return Ok(());
            }
        };
        let ticket_id = self.task.ticket_id.clone();

        let priority_purchase_time = self.task.priority_purchase_time; // 优先购时长分钟

        info!("{}, 正在获取演唱会信息...", self.task.nickname);
        let ticket_info = match self.get_ticket_info(ticket_id.clone()).await {
            Ok(info) => info,
            Err(e) => {
                info!("{}, 获取演唱会信息失败, {:?}", self.task.nickname, e);
                return Err(e);
            }
        };

        if ticket_info
            .detail_view_component_map
            .item
            .item
            .buy_btn_text
            .contains("不支持")
        {
            info!("该渠道不支持购买, 请使用APP购票!");
            return Ok(());
        }

        let ticket_name = self.task.ticket_name.clone();

        let perform_name = self.task.ticket_perform_name.clone();

        let sku_id = self.task.ticket_perform_sku_id.clone();
        let sku_name = self.task.ticket_perform_sku_name.clone();
        let item_id = self.task.ticket_id.clone();

        let start_time_str = ticket_info
            .detail_view_component_map
            .item
            .item
            .sell_start_time_str;

        let mut start_timestamp = ticket_info
            .detail_view_component_map
            .item
            .item
            .sell_start_timestamp
            .parse::<i64>()?;

        if self.task.request_time_offset > 0 {
            start_timestamp += self.task.request_time_offset;
        }

        if priority_purchase_time > 0 {
            start_timestamp += priority_purchase_time * 60 * 1000;
        }

        let date_time = Local.timestamp_millis_opt(start_timestamp).unwrap();

        println!(
            "\n\t账号昵称: {}
            \n\t门票名称: {}
            \n\t场次名称: {}
            \n\t票档名称: {}
            \n\t购票数量: {}
            \n\t官方开售时间: {}
            \n\t实际抢票时间(=官方开售时间 + 请求时间偏移量:{}毫秒 + 优先购时长:{}分钟):{}",
            user_info.nickname,
            ticket_name,
            perform_name,
            sku_name,
            self.task.ticket_num,
            start_time_str,
            self.task.request_time_offset,
            self.task.priority_purchase_time,
            date_time.format("%Y-%m-%d %H:%M:%S.%3f")
        );

        let local: DateTime<Local> = Local::now();
        let current_timestamp = local.timestamp_millis();

        match current_timestamp > start_timestamp {
            true => {
                if let Err(e) = self.buy_it_now(&item_id, &sku_id).await {
                    error!("{}", e.to_string());
                }
            }
            false => {
                let res = self.wait_for_buy(start_timestamp, &item_id, &sku_id).await;
                match res {
                    Ok(is_succes) => {
                        if is_succes {
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("退出") {
                            return Ok(());
                        }
                    }
                };
            }
        };
        Ok(())
    }

    // 立即购买
    pub async fn buy_it_now(&self, item_id: &String, sku_id: &String) -> Result<bool> {
        self.multiple_buy_attempts(item_id, sku_id, None).await
    }

    // 等待开售
    pub async fn wait_for_buy(
        &self,
        start_timestamp: i64,
        item_id: &String,
        sku_id: &String,
    ) -> Result<bool> {
        let (s, r) = async_channel::unbounded::<bool>();

        let interval = rand_i64(30);
        let earliest_submit_time = 0;

        info!("{}, 等待开抢...", self.task.nickname);

        // 轮询等待开抢
        loop {
            tokio::select! {
                _ = signal::ctrl_c() => {
                    return Err(anyhow!("{}, 停止抢票任务...", self.task.nickname));
                }

                _ = tokio::time::sleep(Duration::from_millis(interval)) => {
                    let local: DateTime<Local> = Local::now();
                    let millis = local.timestamp_millis();
                    let time_left_millis = start_timestamp - millis;
                    if time_left_millis <= earliest_submit_time {
                        let _ = s.send(true).await;
                    }else{
                        let (hours, minutes, seconds) = self.ms_to_hms(time_left_millis);
                        print!("\r\t开抢倒计时:{}小时:{}分钟:{:.3}秒\t", hours, minutes, seconds);
                        let _ = io::stdout().flush();

                    }

                }
                _ = r.recv() => {
                    return self.multiple_buy_attempts(item_id, sku_id, None).await
                }
            }
        }
    }
}
