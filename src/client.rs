use std::{
    io::{self, Write},
    time::Duration,
};

use crate::{
    clients::{dm::DmClient, login::LoginClient},
    errors::ClientError,
    models::{
        perform::{PerformForm, PerformInfo, PerformItem, PerformParams, SkuItem},
        task::Task,
        ticket::{
            GetTicketListForm, GetTicketListParams, Ticket, TicketInfo, TicketInfoForm,
            TicketInfoParams, TicketList,
        },
    },
    ticket::DmTicket,
};
use anyhow::Result;
use chrono::{Local, TimeZone};

use log::{debug, error, info};
use terminal_menu::{button, label, menu, mut_menu, numeric, run};
use thirtyfour::{
    cookie::SameSite, prelude::ElementQueryable, By, Cookie, DesiredCapabilities, WebDriver,
};

pub struct Client {
    webdriver_url: String,
    client: LoginClient,
}

impl Client {
    pub async fn new(webdriver_url: String) -> Result<Self> {
        Ok(Self {
            webdriver_url,
            client: LoginClient::new().await?,
        })
    }

    pub async fn qrcode_login(&self) -> Result<String> {
        info!("正在获取二维码...\n");
        let qrcode_data = match self.client.generate_qrcode().await {
            Ok(data) => {
                debug!("Get qrcode data:{:?}", data);
                data
            }
            Err(e) => {
                error!("Fail to get qrcode data, error:{:?}", e);
                return Err(e);
            }
        };

        let qrcode = match self.client.get_qrcode(qrcode_data.code_content).await {
            Ok(code) => {
                debug!("success to get qrcode!");
                code
            }
            Err(e) => {
                error!("Fail to get qrcode, error:{:?}", e);
                return Err(e);
            }
        };

        println!("{}\n", qrcode.to_str());

        let t = qrcode_data.t;
        let ck = qrcode_data.ck.clone();

        let max_times = 60 * 5;

        for i in 0..max_times {
            let qrcode_scan_status = self.client.get_login_result(t, ck.clone()).await?;

            match qrcode_scan_status.qrcode_status.as_str() {
                "NEW" => {
                    print!("\r请使用大麦APP扫码, 倒计时:{}秒\t", max_times - i);
                    let _ = io::stdout().flush();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                "SCANED" => {
                    print!("\r##########请点击确认登录#######\t\t");
                    let _ = io::stdout().flush();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                "CONFIRMED" => {
                    let cookie2 = qrcode_scan_status.cookie2.unwrap();
                    let return_url = qrcode_scan_status.return_url.unwrap();
                    let st = qrcode_scan_status.st.unwrap();
                    let _ = self.client.get_cookie(&cookie2, return_url, st).await?;
                    println!("\r\n扫码登录成功!");
                    return Ok(cookie2);
                }
                "EXPIRED" => {
                    print!("\r\t二维码已过期, 请重新执行程序...");
                    return Err(ClientError::LoginFailed.into());
                }
                _ => {
                    error!("未知状态:{:?}, 退出...", qrcode_scan_status);
                    return Err(ClientError::LoginFailed.into());
                }
            }
        }

        info!("二维码已过期, 请重新执行程序...");

        Err(ClientError::LoginFailed.into())
    }

    pub async fn get_driver(&self, webdriver_url: String) -> Result<WebDriver> {
        let mut caps = DesiredCapabilities::chrome();
        caps.set_disable_dev_shm_usage()?;
        caps.set_headless()?;
        caps.set_disable_gpu()?;
        caps.set_disable_web_security()?;
        caps.set_ignore_certificate_errors()?;
        caps.add_chrome_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_chrome_arg("--disable-logging")?;
        //caps.add_chrome_arg("--blink-settings=imagesEnabled=false")?;
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
        let driver: WebDriver = WebDriver::new(&webdriver_url, caps.clone())
            .await
            .map_err(|_| ClientError::WebdriverConnectionError)?;
        Ok(driver)
    }

    pub async fn login(&self) -> Result<(String, String)> {
        let cookie2 = self.qrcode_login().await?;

        info!("正在获取cookie...");
        let driver = self.get_driver(self.webdriver_url.clone()).await?;
        let mut c = Cookie::new("cookie2", cookie2);
        c.set_domain("damai.cn");
        c.set_path("/");
        c.set_same_site(Some(SameSite::Lax));

        driver.goto("https://m.damai.cn/").await?;
        let _ = driver.add_cookie(c).await;

        let h5_url = "https://m.damai.cn/damai/mine/my/index.html?spm=a2o71.home.top.duserinfo";
        driver.goto(h5_url).await?;

        let css = r#"body > div.my > div.my-hd > div.user-name > div.nickname"#;
        let _ = driver
            .query(By::Css(css))
            .wait(Duration::from_secs(10), Duration::from_millis(100))
            .first()
            .await;
        let cookies = driver.get_all_cookies().await?;

        let mut cookie_string = String::new();

        for item in cookies {
            if item.name().starts_with("_m_h5_tk") {
                continue;
            }
            cookie_string.push_str(&format!("{}={};", item.name(), item.value()));
        }

        let _ = driver.quit().await;

        Ok((cookie_string, "".to_string()))
    }

    // 获取演唱会ID
    pub async fn get_ticket_id(&self) -> Result<Ticket> {
        let dm = DmClient::new(None, None).await?;
        let url = "https://mtop.damai.cn/h5/mtop.damai.wireless.search.broadcast.list/1.0/";
        let params = GetTicketListParams::build()?;
        let form = GetTicketListForm::build()?;

        let res = dm.request(url, params, form).await?;

        // 今日必抢
        let today_ticket_list: TicketList = serde_json::from_value(res.data["modules"][0].clone())?;

        // 即将开抢
        let ticket_list: TicketList = serde_json::from_value(res.data["modules"][1].clone())?;

        let mut tickets: Vec<Ticket> = Vec::new();

        for ticket in today_ticket_list.items {
            if !ticket.category_name.contains("演唱会") {
                continue;
            }
            tickets.push(ticket);
        }

        for ticket in ticket_list.items {
            if !ticket.category_name.contains("演唱会") {
                continue;
            }
            tickets.push(ticket);
        }

        let mut select_list = vec![label("请选择演唱会:")];
        for ticket in tickets.iter() {
            let date_time = Local.timestamp_millis_opt(ticket.sale_time as i64).unwrap();
            select_list.push(button(format!(
                "{}, 开抢时间:{}",
                ticket.ticket_name,
                date_time.format("%Y-%m-%d %H:%M:%S")
            )));
        }
        let m = menu(select_list);
        run(&m);
        let index = mut_menu(&m).selected_item_index() - 1;

        Ok(tickets[index].clone())
    }

    pub async fn get_perform(&self, ticket_id: &String) -> Result<PerformItem> {
        let dm = DmClient::new(None, None).await?;

        let url = "https://mtop.damai.cn/h5/mtop.alibaba.damai.detail.getdetail/1.2";

        let params = TicketInfoParams::build()?;

        let data = TicketInfoForm::build(ticket_id)?;

        let res = dm.request(url, params, data).await?;

        let ticket_info: TicketInfo =
            serde_json::from_str(res.data["result"].clone().as_str().unwrap())?;

        let perform_list = ticket_info
            .detail_view_component_map
            .item
            .item
            .perform_bases;

        let mut performs: Vec<PerformItem> = Vec::new();

        for perform in perform_list.iter() {
            for item in perform.performs.iter() {
                performs.push(PerformItem {
                    perfrom_name: item.perform_name.clone(),
                    perform_id: item.perform_id.clone(),
                })
            }
        }

        let mut select_list = vec![label("请选择场次:")];

        for perform in performs.iter() {
            select_list.push(button(perform.perfrom_name.clone()));
        }

        let m = menu(select_list);
        run(&m);

        let index = mut_menu(&m).selected_item_index() - 1;

        Ok(performs[index].clone())
    }

    pub async fn get_sku(&self, ticket_id: String, perfrom_id: String) -> Result<SkuItem> {
        let dm = DmClient::new(None, None).await?;

        let url = "https://mtop.damai.cn/h5/mtop.alibaba.detail.subpage.getdetail/2.0/";

        let params = PerformParams::build()?;

        let data = PerformForm::build(&ticket_id, &perfrom_id)?;

        let res = dm.request(url, params, data).await?;

        let perform_info: PerformInfo = serde_json::from_str(res.data["result"].as_str().unwrap())?;

        let mut skus: Vec<SkuItem> = vec![];
        for item in perform_info.perform.sku_list.iter() {
            skus.push(SkuItem {
                sku_id: item.sku_id.clone(),
                sku_name: item.price_name.clone(),
            })
        }

        let mut select_list = vec![label("请选择票档:")];
        for sku in skus.iter() {
            select_list.push(button(sku.sku_name.clone()));
        }

        let m = menu(select_list);
        run(&m);
        let index = mut_menu(&m).selected_item_index() - 1;

        Ok(skus[index].clone())
    }

    pub async fn run(&self) -> Result<()> {
        let m = menu(vec![
            label("请选择登录方式:"),
            button("1.扫码登录"),
            button("2.输入cookie"),
        ]);
        run(&m);

        let selected = mut_menu(&m).selected_item_index();

        let (cookie, nickname) = match selected {
            1 => {
                let (cookie, nickname) =
                    self.login().await.map_err(|_| ClientError::LoginFailed)?;
                (cookie, nickname)
            }
            2 => {
                let mut cookie = String::new();
                println!("\r\n请输入cookie:");
                let _ = std::io::stdin().read_line(&mut cookie).expect("输入错误!");
                (cookie, "xxx".to_string())
            }
            _ => {
                panic!("error: unexpected");
            }
        };

        if !cookie.contains("cookie2") {
            return Err(ClientError::CookieError.into());
        }
        info!("正在获取演唱会ID");
        let ticket = self.get_ticket_id().await?;

        let perform = self.get_perform(&ticket.ticket_id.to_string()).await?;

        let sku = self
            .get_sku(ticket.ticket_id.to_string(), perform.perform_id.to_string())
            .await?;

        let m = menu(vec![
            numeric(
                "购票数量",
                1.0, //default
                Some(1.0),
                Some(1.0),
                Some(4.0),
            ),
            button("确定"),
        ]);
        run(&m);
        let ticket_num = mut_menu(&m).numeric_value("购票数量");

        let m = menu(vec![
            numeric("重试次数", 5.0, Some(1.0), None, Some(10.0)),
            button("确定"),
        ]);
        run(&m);
        let retry_times = mut_menu(&m).numeric_value("重试次数");

        let m = menu(vec![
            numeric("重试间隔", 100.0, Some(10.0), None, Some(1000.0)),
            button("确定"),
        ]);
        run(&m);
        let retry_interval = mut_menu(&m).numeric_value("重试间隔");

        let m = menu(vec![
            numeric("生成-提交订单间隔(毫秒)", 30.0, Some(10.0), Some(0.0), None),
            button("确定"),
        ]);
        run(&m);
        let wati_for_submit_interval = mut_menu(&m).numeric_value("生成-提交订单间隔(毫秒)");

        let m = menu(vec![
            numeric(
                "请求时间偏移量(毫秒)",
                0.0,
                Some(10.0),
                Some(-100.0),
                Some(1000.0),
            ),
            button("确定"),
        ]);
        run(&m);
        let request_time_offset = mut_menu(&m).numeric_value("请求时间偏移量(毫秒)");

        let m = menu(vec![
            numeric("优先购时长(分钟)", 0.0, Some(20.0), Some(0.0), Some(60.0)),
            button("确定"),
        ]);
        run(&m);
        let priority_purchase_time = mut_menu(&m).numeric_value("优先购时长(分钟)");

        let task = Task {
            nickname,
            ticket_id: ticket.ticket_id.to_string(),
            ticket_name: ticket.ticket_name.to_string(),
            ticket_perform_id: perform.perform_id.to_string(),
            ticket_perform_name: perform.perfrom_name,
            ticket_perform_sku_id: sku.sku_id,
            ticket_perform_sku_name: sku.sku_name,
            ticket_num: ticket_num as usize,
            priority_purchase_time: priority_purchase_time as i64,
            request_time_offset: request_time_offset as i64,
            retry_interval: retry_interval as u64,
            retry_times: retry_times as u64,
            wait_for_submit_interval: wati_for_submit_interval as u64,
            real_names: vec![],
        };

        let mut app = DmTicket::new(cookie, task).await?;
        app.run().await?;

        Ok(())
    }
}
